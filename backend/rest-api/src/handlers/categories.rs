use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, put};
use axum::{Json, Router};
use uuid::Uuid;
use validator::Validate;

use crate::AppState;
use crate::dto::categories::{CategoryResponse, CreateCategoryRequest, UpdateCategoryRequest};
use crate::errors::AppError;
use crate::middleware::AuthUser;

fn validate_request(req: &impl Validate) -> Result<(), AppError> {
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_categories).post(create_category))
        .route("/{id}", put(update_category).delete(delete_category))
}

#[tracing::instrument(skip(state))]
async fn list_categories(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<Vec<CategoryResponse>>, AppError> {
    let response = state.categories.list_categories(auth_user.user_id).await?;

    Ok(Json(response))
}

#[tracing::instrument(skip(state, req))]
async fn create_category(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<CreateCategoryRequest>,
) -> Result<(StatusCode, Json<CategoryResponse>), AppError> {
    validate_request(&req)?;

    let response = state
        .categories
        .create_category(auth_user.user_id, &req.name, req.icon.as_deref())
        .await?;

    Ok((StatusCode::CREATED, Json(response)))
}

#[tracing::instrument(skip(state, req))]
async fn update_category(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCategoryRequest>,
) -> Result<Json<CategoryResponse>, AppError> {
    validate_request(&req)?;

    let response = state
        .categories
        .update_category(
            id,
            auth_user.user_id,
            req.name.as_deref(),
            req.icon.as_deref(),
            req.position,
        )
        .await?;

    Ok(Json(response))
}

#[tracing::instrument(skip(state))]
async fn delete_category(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state
        .categories
        .delete_category(id, auth_user.user_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    fn make_create(name: &str) -> CreateCategoryRequest {
        CreateCategoryRequest {
            name: name.into(),
            icon: None,
        }
    }

    // ── CreateCategoryRequest ────────────────────────────────────────

    #[test]
    fn create_rejects_empty_name() {
        assert!(make_create("").validate().is_err());
    }

    #[test]
    fn create_accepts_min_name() {
        assert!(make_create("x").validate().is_ok());
    }

    #[test]
    fn create_accepts_100_char_name() {
        assert!(make_create(&"a".repeat(100)).validate().is_ok());
    }

    #[test]
    fn create_rejects_101_char_name() {
        assert!(make_create(&"a".repeat(101)).validate().is_err());
    }

    #[test]
    fn create_accepts_50_char_icon() {
        let req = CreateCategoryRequest {
            name: "test".into(),
            icon: Some("a".repeat(50)),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn create_rejects_51_char_icon() {
        let req = CreateCategoryRequest {
            name: "test".into(),
            icon: Some("a".repeat(51)),
        };
        assert!(req.validate().is_err());
    }

    // ── UpdateCategoryRequest ────────────────────────────────────────

    #[test]
    fn update_accepts_empty_body() {
        let req = UpdateCategoryRequest {
            name: None,
            icon: None,
            position: None,
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn update_rejects_empty_name() {
        let req = UpdateCategoryRequest {
            name: Some("".into()),
            icon: None,
            position: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn update_rejects_101_char_name() {
        let req = UpdateCategoryRequest {
            name: Some("a".repeat(101)),
            icon: None,
            position: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn update_accepts_valid_partial() {
        let req = UpdateCategoryRequest {
            name: Some("renamed".into()),
            icon: None,
            position: None,
        };
        assert!(req.validate().is_ok());
    }
}
