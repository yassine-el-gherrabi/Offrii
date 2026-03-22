mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use tower::ServiceExt;

// ── Helpers ──────────────────────────────────────────────────────────

async fn login(
    app: &common::TestApp,
    email: &str,
    password: &str,
) -> (StatusCode, serde_json::Value) {
    let body = serde_json::json!({ "identifier": email, "password": password });
    app.post_json("/auth/login", &body).await
}

async fn count_connection_logs(app: &common::TestApp, email: &str) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM connection_logs cl \
         JOIN users u ON u.id = cl.user_id \
         WHERE u.email = $1",
    )
    .bind(email)
    .fetch_one(&app.db)
    .await
    .unwrap()
}

async fn get_last_active(
    app: &common::TestApp,
    email: &str,
) -> Option<chrono::DateTime<chrono::Utc>> {
    sqlx::query_scalar::<_, Option<chrono::DateTime<chrono::Utc>>>(
        "SELECT last_active_at FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_one(&app.db)
    .await
    .unwrap()
}

// ── Connection Log Tests ─────────────────────────────────────────────

#[tokio::test]
async fn register_creates_connection_log() {
    let app = common::TestApp::new().await;
    app.setup_user("user@test.com", common::TEST_PASSWORD).await;

    let count = count_connection_logs(&app, "user@test.com").await;
    assert_eq!(count, 1, "registration should create one connection log");
}

#[tokio::test]
async fn login_creates_connection_log() {
    let app = common::TestApp::new().await;
    app.setup_user("user@test.com", common::TEST_PASSWORD).await;

    // Login
    let (status, _) = login(&app, "user@test.com", common::TEST_PASSWORD).await;
    assert_eq!(status, StatusCode::OK);

    // Should have 2 logs: 1 from register + 1 from login
    let count = count_connection_logs(&app, "user@test.com").await;
    assert_eq!(count, 2, "login should create an additional connection log");
}

#[tokio::test]
async fn refresh_does_not_create_connection_log() {
    let app = common::TestApp::new().await;
    let reg = app.setup_user("user@test.com", common::TEST_PASSWORD).await;
    let refresh_token = reg["tokens"]["refresh_token"].as_str().unwrap();

    // Refresh
    let body = serde_json::json!({ "refresh_token": refresh_token });
    let (status, _) = app.post_json("/auth/refresh", &body).await;
    assert_eq!(status, StatusCode::OK);

    // Should still have only 1 log (from register, not from refresh)
    let count = count_connection_logs(&app, "user@test.com").await;
    assert_eq!(count, 1, "refresh should NOT create a connection log");
}

#[tokio::test]
async fn connection_log_stores_user_agent() {
    let app = common::TestApp::new().await;

    // Register with a custom User-Agent header
    let body = serde_json::json!({
        "email": "ua@test.com",
        "password": common::TEST_PASSWORD,
        "terms_accepted": true,
    });
    let req = Request::builder()
        .method("POST")
        .uri("/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::USER_AGENT, "Offrii/1.0 iPhone iOS/17.5")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Check the log has the user agent
    let ua: String = sqlx::query_scalar(
        "SELECT cl.user_agent FROM connection_logs cl \
         JOIN users u ON u.id = cl.user_id \
         WHERE u.email = $1 LIMIT 1",
    )
    .bind("ua@test.com")
    .fetch_one(&app.db)
    .await
    .unwrap();

    assert_eq!(ua, "Offrii/1.0 iPhone iOS/17.5");
}

#[tokio::test]
async fn connection_log_purge_removes_old_entries() {
    let app = common::TestApp::new().await;
    app.setup_user("user@test.com", common::TEST_PASSWORD).await;

    // Manually backdate the log to 13 months ago
    sqlx::query(
        "UPDATE connection_logs SET created_at = NOW() - INTERVAL '13 months' \
         WHERE user_id = (SELECT id FROM users WHERE email = $1)",
    )
    .bind("user@test.com")
    .execute(&app.db)
    .await
    .unwrap();

    // Run the purge query
    let result =
        sqlx::query("DELETE FROM connection_logs WHERE created_at < NOW() - INTERVAL '12 months'")
            .execute(&app.db)
            .await
            .unwrap();

    assert!(result.rows_affected() > 0, "should purge old logs");

    let count = count_connection_logs(&app, "user@test.com").await;
    assert_eq!(count, 0, "no logs should remain after purge");
}

// ── last_active_at Tests ─────────────────────────────────────────────

#[tokio::test]
async fn register_sets_last_active_at() {
    let app = common::TestApp::new().await;
    app.setup_user("user@test.com", common::TEST_PASSWORD).await;

    let last_active = get_last_active(&app, "user@test.com").await;
    assert!(
        last_active.is_some(),
        "last_active_at should be set after registration"
    );
}

#[tokio::test]
async fn authenticated_request_updates_last_active_at() {
    let app = common::TestApp::new().await;
    let token = app
        .setup_user_token("user@test.com", common::TEST_PASSWORD)
        .await;

    // Get user_id
    let user_id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind("user@test.com")
        .fetch_one(&app.db)
        .await
        .unwrap();

    // Manually set last_active_at to 1 hour ago to bypass throttle
    sqlx::query("UPDATE users SET last_active_at = NOW() - INTERVAL '1 hour' WHERE email = $1")
        .bind("user@test.com")
        .execute(&app.db)
        .await
        .unwrap();

    // Clear the Redis throttle key so the middleware will update
    if let Ok(mut conn) = app.redis.get_multiplexed_async_connection().await {
        let _: Result<(), _> = redis::cmd("DEL")
            .arg(format!("active:{user_id}"))
            .query_async(&mut conn)
            .await;
    }

    let before = get_last_active(&app, "user@test.com").await.unwrap();

    // Make an authenticated request (test router nests users under /users)
    let (status, _) = app.get_with_auth("/users/profile", &token).await;
    assert_eq!(status, StatusCode::OK);

    // Small delay for the spawned task to complete
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let after = get_last_active(&app, "user@test.com").await.unwrap();
    assert!(
        after > before,
        "last_active_at should be updated after authenticated request"
    );
}

// ── Inactive Account Tests ───────────────────────────────────────────

#[tokio::test]
async fn inactive_account_warning_query_finds_old_users() {
    let app = common::TestApp::new().await;
    app.setup_user("old@test.com", common::TEST_PASSWORD).await;
    app.setup_user("active@test.com", common::TEST_PASSWORD)
        .await;

    // Make old@test.com inactive for 24 months
    sqlx::query("UPDATE users SET last_active_at = NOW() - INTERVAL '24 months' WHERE email = $1")
        .bind("old@test.com")
        .execute(&app.db)
        .await
        .unwrap();

    // Query: find users inactive > 23 months without notice
    let inactive: Vec<(uuid::Uuid, String)> = sqlx::query_as(
        "SELECT id, email FROM users \
         WHERE last_active_at < NOW() - INTERVAL '23 months' \
         AND inactivity_notice_sent_at IS NULL",
    )
    .fetch_all(&app.db)
    .await
    .unwrap();

    assert_eq!(inactive.len(), 1);
    assert_eq!(inactive[0].1, "old@test.com");
}

#[tokio::test]
async fn inactive_account_deletion_query_respects_notice() {
    let app = common::TestApp::new().await;
    app.setup_user("doomed@test.com", common::TEST_PASSWORD)
        .await;

    // Make inactive 25 months + notice sent 31 days ago
    sqlx::query(
        "UPDATE users SET \
         last_active_at = NOW() - INTERVAL '25 months', \
         inactivity_notice_sent_at = NOW() - INTERVAL '31 days' \
         WHERE email = $1",
    )
    .bind("doomed@test.com")
    .execute(&app.db)
    .await
    .unwrap();

    // Run deletion query
    let result = sqlx::query(
        "DELETE FROM users \
         WHERE inactivity_notice_sent_at IS NOT NULL \
         AND inactivity_notice_sent_at < NOW() - INTERVAL '30 days' \
         AND last_active_at < NOW() - INTERVAL '24 months'",
    )
    .execute(&app.db)
    .await
    .unwrap();

    assert_eq!(result.rows_affected(), 1, "should delete the inactive user");
}

#[tokio::test]
async fn reactivated_user_not_deleted() {
    let app = common::TestApp::new().await;
    app.setup_user("saved@test.com", common::TEST_PASSWORD)
        .await;

    // Set notice sent 31 days ago, BUT user reconnected recently
    sqlx::query(
        "UPDATE users SET \
         last_active_at = NOW(), \
         inactivity_notice_sent_at = NOW() - INTERVAL '31 days' \
         WHERE email = $1",
    )
    .bind("saved@test.com")
    .execute(&app.db)
    .await
    .unwrap();

    // Run deletion query
    let result = sqlx::query(
        "DELETE FROM users \
         WHERE inactivity_notice_sent_at IS NOT NULL \
         AND inactivity_notice_sent_at < NOW() - INTERVAL '30 days' \
         AND last_active_at < NOW() - INTERVAL '24 months'",
    )
    .execute(&app.db)
    .await
    .unwrap();

    assert_eq!(
        result.rows_affected(),
        0,
        "should NOT delete user who reconnected after warning"
    );

    // Verify user still exists
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind("saved@test.com")
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(count, 1);
}
