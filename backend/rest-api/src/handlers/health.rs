use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Serialize;

use crate::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub db: String,
    pub redis: String,
}

#[tracing::instrument(skip(state))]
pub async fn health_check(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    let db_ok = sqlx::query("SELECT 1").fetch_one(&state.db).await.is_ok();

    let redis_ok = check_redis(&state.redis).await;

    let (status_code, status) = if db_ok && redis_ok {
        (StatusCode::OK, "ok")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "degraded")
    };

    let response = HealthResponse {
        status: status.to_string(),
        db: if db_ok {
            "connected".to_string()
        } else {
            "disconnected".to_string()
        },
        redis: if redis_ok {
            "connected".to_string()
        } else {
            "disconnected".to_string()
        },
    };

    (status_code, Json(response))
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
    fn health_response_serializes_correctly() {
        let response = HealthResponse {
            status: "ok".to_string(),
            db: "connected".to_string(),
            redis: "connected".to_string(),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["status"], "ok");
        assert_eq!(json["db"], "connected");
        assert_eq!(json["redis"], "connected");
    }

    #[test]
    fn health_response_degraded_serializes() {
        let response = HealthResponse {
            status: "degraded".to_string(),
            db: "connected".to_string(),
            redis: "disconnected".to_string(),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["status"], "degraded");
        assert_eq!(json["redis"], "disconnected");
    }
}
