use std::sync::Arc;

use uuid::Uuid;

use crate::traits::ReminderService;

/// Run the reminder job with a distributed lock to prevent concurrent execution.
///
/// Lock strategy: SET NX EX with a 5-minute TTL. If the lock is acquired,
/// execute the hourly tick, then delete the lock with a CAS check (ownership).
/// If Redis is down, fail-open (skip this tick, retry next hour).
pub async fn run(svc: Arc<dyn ReminderService>, redis: redis::Client) {
    let lock_key = "reminder_job:lock";
    let lock_value = Uuid::new_v4().to_string();

    let mut conn = match redis.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(error = %e, "reminder_job: redis unavailable, skipping tick");
            return;
        }
    };

    // Try to acquire lock: SET key value NX EX 300
    let acquired: bool = match redis::cmd("SET")
        .arg(lock_key)
        .arg(&lock_value)
        .arg("NX")
        .arg("EX")
        .arg(300_u64)
        .query_async(&mut conn)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = %e, "reminder_job: failed to acquire lock");
            return;
        }
    };

    if !acquired {
        tracing::debug!("reminder_job: lock held by another instance, skipping");
        return;
    }

    tracing::info!("reminder_job: lock acquired, executing tick");

    svc.execute_hourly_tick().await;

    // Release lock with CAS: only delete if we own it
    let lua_script = r#"
        if redis.call("GET", KEYS[1]) == ARGV[1] then
            return redis.call("DEL", KEYS[1])
        else
            return 0
        end
    "#;

    let _: Result<i32, _> = redis::cmd("EVAL")
        .arg(lua_script)
        .arg(1_i32)
        .arg(lock_key)
        .arg(&lock_value)
        .query_async(&mut conn)
        .await;

    tracing::info!("reminder_job: tick completed, lock released");
}
