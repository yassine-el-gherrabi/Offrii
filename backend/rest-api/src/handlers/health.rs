use std::time::Duration;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Serialize;
use tokio::time::timeout;

use crate::AppState;

const HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub db: String,
    pub redis: String,
}

#[tracing::instrument(skip(state))]
pub async fn health_check(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    let db_ok = timeout(
        HEALTH_CHECK_TIMEOUT,
        sqlx::query("SELECT 1").fetch_one(&state.db),
    )
    .await
    .is_ok_and(|r| r.is_ok());

    let redis_ok = timeout(HEALTH_CHECK_TIMEOUT, check_redis(&state.redis))
        .await
        .unwrap_or(false);

    let (status_code, response) = build_response(db_ok, redis_ok);
    (status_code, Json(response))
}

fn build_response(db_ok: bool, redis_ok: bool) -> (StatusCode, HealthResponse) {
    let (status_code, status) = if db_ok && redis_ok {
        (StatusCode::OK, "ok")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "degraded")
    };

    let response = HealthResponse {
        status: status.to_string(),
        db: connection_status(db_ok),
        redis: connection_status(redis_ok),
    };

    (status_code, response)
}

fn connection_status(ok: bool) -> String {
    if ok { "connected" } else { "disconnected" }.to_string()
}

async fn check_redis(client: &redis::Client) -> bool {
    let Ok(mut conn) = client.get_multiplexed_async_connection().await else {
        return false;
    };
    redis::cmd("PING")
        .query_async::<String>(&mut conn)
        .await
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_healthy_returns_ok() {
        let (status, response) = build_response(true, true);
        assert_eq!(status, StatusCode::OK);
        assert_eq!(response.status, "ok");
        assert_eq!(response.db, "connected");
        assert_eq!(response.redis, "connected");
    }

    #[test]
    fn db_down_returns_degraded() {
        let (status, response) = build_response(false, true);
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(response.status, "degraded");
        assert_eq!(response.db, "disconnected");
        assert_eq!(response.redis, "connected");
    }

    #[test]
    fn redis_down_returns_degraded() {
        let (status, response) = build_response(true, false);
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(response.status, "degraded");
        assert_eq!(response.db, "connected");
        assert_eq!(response.redis, "disconnected");
    }

    #[test]
    fn all_down_returns_degraded() {
        let (status, response) = build_response(false, false);
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(response.status, "degraded");
        assert_eq!(response.db, "disconnected");
        assert_eq!(response.redis, "disconnected");
    }
}
