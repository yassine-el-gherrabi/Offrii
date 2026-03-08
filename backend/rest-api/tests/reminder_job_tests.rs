mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use async_trait::async_trait;

use rest_api::jobs::reminder_job;
use rest_api::traits::ReminderService;

/// Spy that counts how many times `execute_hourly_tick` was called.
struct SpyReminderService {
    tick_count: AtomicU32,
}

impl SpyReminderService {
    fn new() -> Self {
        Self {
            tick_count: AtomicU32::new(0),
        }
    }

    fn tick_count(&self) -> u32 {
        self.tick_count.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl ReminderService for SpyReminderService {
    async fn execute_hourly_tick(&self) {
        self.tick_count.fetch_add(1, Ordering::SeqCst);
    }
}

#[tokio::test]
async fn run_acquires_lock_and_executes_tick() {
    let app = common::TestApp::new().await;
    let svc = Arc::new(SpyReminderService::new());

    reminder_job::run(svc.clone(), app.redis.clone()).await;

    assert_eq!(svc.tick_count(), 1, "tick should have executed once");

    // Lock should be released after run completes
    let mut conn = app.redis.get_multiplexed_async_connection().await.unwrap();
    let lock_exists: bool = redis::cmd("EXISTS")
        .arg("reminder_job:lock")
        .query_async(&mut conn)
        .await
        .unwrap();
    assert!(!lock_exists, "lock should be released after run");
}

#[tokio::test]
async fn run_skips_when_lock_already_held() {
    let app = common::TestApp::new().await;

    // Pre-set the lock to simulate another instance holding it
    let mut conn = app.redis.get_multiplexed_async_connection().await.unwrap();
    let _: () = redis::cmd("SET")
        .arg("reminder_job:lock")
        .arg("other-instance")
        .arg("EX")
        .arg(300_u64)
        .query_async(&mut conn)
        .await
        .unwrap();

    let svc = Arc::new(SpyReminderService::new());
    reminder_job::run(svc.clone(), app.redis.clone()).await;

    assert_eq!(
        svc.tick_count(),
        0,
        "tick should NOT execute when lock is held"
    );

    // Original lock should still be intact (not deleted by CAS)
    let val: Option<String> = redis::cmd("GET")
        .arg("reminder_job:lock")
        .query_async(&mut conn)
        .await
        .unwrap();
    assert_eq!(
        val.as_deref(),
        Some("other-instance"),
        "foreign lock should remain"
    );
}

#[tokio::test]
async fn run_skips_when_redis_unavailable() {
    // Create a client pointing to a non-existent Redis
    let bad_redis = redis::Client::open("redis://127.0.0.1:1").unwrap();
    let svc = Arc::new(SpyReminderService::new());

    reminder_job::run(svc.clone(), bad_redis).await;

    assert_eq!(
        svc.tick_count(),
        0,
        "tick should NOT execute when redis is down"
    );
}

#[tokio::test]
async fn run_twice_sequentially_both_execute() {
    let app = common::TestApp::new().await;
    let svc = Arc::new(SpyReminderService::new());

    reminder_job::run(svc.clone(), app.redis.clone()).await;
    reminder_job::run(svc.clone(), app.redis.clone()).await;

    assert_eq!(svc.tick_count(), 2, "both sequential runs should execute");
}
