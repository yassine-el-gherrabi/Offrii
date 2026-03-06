use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use uuid::Uuid;
use validator::Validate;

use crate::AppState;
use crate::dto::items::{
    CreateItemRequest, ItemResponse, ItemsListResponse, ListItemsQuery, UpdateItemRequest,
};
use crate::errors::AppError;
use crate::middleware::AuthUser;

fn validate_request(req: &impl Validate) -> Result<(), AppError> {
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_items).post(create_item))
        .route("/{id}", get(get_item).put(update_item).delete(delete_item))
}

#[tracing::instrument(skip(state, req))]
async fn create_item(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<CreateItemRequest>,
) -> Result<(StatusCode, Json<ItemResponse>), AppError> {
    validate_request(&req)?;

    let response = state
        .items
        .create_item(
            auth_user.user_id,
            &req.name,
            req.description.as_deref(),
            req.url.as_deref(),
            req.estimated_price,
            req.priority,
            req.category_id,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(response)))
}

#[tracing::instrument(skip(state))]
async fn list_items(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(query): Query<ListItemsQuery>,
) -> Result<Json<ItemsListResponse>, AppError> {
    let response = state.items.list_items(auth_user.user_id, &query).await?;

    Ok(Json(response))
}

#[tracing::instrument(skip(state))]
async fn get_item(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ItemResponse>, AppError> {
    let response = state.items.get_item(id, auth_user.user_id).await?;

    Ok(Json(response))
}

#[tracing::instrument(skip(state, req))]
async fn update_item(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateItemRequest>,
) -> Result<Json<ItemResponse>, AppError> {
    validate_request(&req)?;

    let response = state
        .items
        .update_item(
            id,
            auth_user.user_id,
            req.name.as_deref(),
            req.description.as_deref(),
            req.url.as_deref(),
            req.estimated_price,
            req.priority,
            req.category_id.map(Some),
            req.status.as_deref(),
        )
        .await?;

    Ok(Json(response))
}

#[tracing::instrument(skip(state))]
async fn delete_item(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.items.delete_item(id, auth_user.user_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;
    use validator::Validate;

    fn make_create(name: &str) -> CreateItemRequest {
        CreateItemRequest {
            name: name.into(),
            description: None,
            url: None,
            estimated_price: None,
            priority: None,
            category_id: None,
        }
    }

    // ── CreateItemRequest ──────────────────────────────────────────

    #[test]
    fn create_rejects_empty_name() {
        assert!(make_create("").validate().is_err());
    }

    #[test]
    fn create_accepts_min_name() {
        assert!(make_create("x").validate().is_ok());
    }

    #[test]
    fn create_accepts_255_char_name() {
        assert!(make_create(&"a".repeat(255)).validate().is_ok());
    }

    #[test]
    fn create_rejects_256_char_name() {
        assert!(make_create(&"a".repeat(256)).validate().is_err());
    }

    #[test]
    fn create_accepts_5000_char_description() {
        let mut req = make_create("x");
        req.description = Some("a".repeat(5000));
        assert!(req.validate().is_ok());
    }

    #[test]
    fn create_rejects_5001_char_description() {
        let mut req = make_create("x");
        req.description = Some("a".repeat(5001));
        assert!(req.validate().is_err());
    }

    #[test]
    fn create_accepts_2048_char_url() {
        let mut req = make_create("x");
        req.url = Some("a".repeat(2048));
        assert!(req.validate().is_ok());
    }

    #[test]
    fn create_rejects_2049_char_url() {
        let mut req = make_create("x");
        req.url = Some("a".repeat(2049));
        assert!(req.validate().is_err());
    }

    #[test]
    fn create_accepts_all_optional_fields() {
        let req = CreateItemRequest {
            name: "test".into(),
            description: Some("desc".into()),
            url: Some("https://example.com".into()),
            estimated_price: Some(Decimal::from_str("9.99").unwrap()),
            priority: Some(1),
            category_id: Some(Uuid::new_v4()),
        };
        assert!(req.validate().is_ok());
    }

    // ── UpdateItemRequest ──────────────────────────────────────────

    #[test]
    fn update_accepts_empty_body() {
        let req = UpdateItemRequest {
            name: None,
            description: None,
            url: None,
            estimated_price: None,
            priority: None,
            category_id: None,
            status: None,
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn update_rejects_empty_name() {
        let req = UpdateItemRequest {
            name: Some("".into()),
            description: None,
            url: None,
            estimated_price: None,
            priority: None,
            category_id: None,
            status: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn update_rejects_256_char_name() {
        let req = UpdateItemRequest {
            name: Some("a".repeat(256)),
            description: None,
            url: None,
            estimated_price: None,
            priority: None,
            category_id: None,
            status: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn update_accepts_valid_partial() {
        let req = UpdateItemRequest {
            name: Some("renamed".into()),
            description: None,
            url: None,
            estimated_price: None,
            priority: None,
            category_id: None,
            status: None,
        };
        assert!(req.validate().is_ok());
    }
}
