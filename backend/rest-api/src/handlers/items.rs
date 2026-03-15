use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use uuid::Uuid;
use validator::Validate;

use crate::AppState;
use crate::dto::items::{
    BatchDeleteRequest, CreateItemRequest, ItemResponse, ListItemsQuery, UpdateItemRequest,
};
use crate::dto::pagination::PaginatedResponse;
use crate::errors::AppError;
use crate::middleware::AuthUser;

fn validate_request(req: &impl Validate) -> Result<(), AppError> {
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_items).post(create_item))
        .route("/batch-delete", axum::routing::post(batch_delete))
        .route("/{id}", get(get_item).put(update_item).delete(delete_item))
        .route(
            "/{id}/claim",
            axum::routing::post(claim_item).delete(unclaim_item),
        )
        .route("/{id}/web-claim", axum::routing::delete(owner_unclaim_web))
}

#[tracing::instrument(skip(state, req))]
async fn create_item(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<CreateItemRequest>,
) -> Result<(StatusCode, Json<ItemResponse>), AppError> {
    validate_request(&req)?;
    req.validate_links().map_err(AppError::BadRequest)?;

    let resolved_links = req.resolved_links();

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
            req.image_url.as_deref(),
            resolved_links.as_deref(),
            req.is_private,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(response)))
}

#[tracing::instrument(skip(state))]
async fn list_items(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(query): Query<ListItemsQuery>,
) -> Result<Json<PaginatedResponse<ItemResponse>>, AppError> {
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
    req.validate_links().map_err(AppError::BadRequest)?;

    let resolved_links = req.resolved_links();

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
            req.image_url.as_deref(),
            resolved_links.as_deref(),
            req.is_private,
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

#[tracing::instrument(skip(state))]
async fn claim_item(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.items.claim_item(id, auth_user.user_id).await?;

    // Best-effort circle events — don't fail the claim if this errors
    if let Err(e) = state.circles.on_item_claimed(id, auth_user.user_id).await {
        tracing::warn!(item_id = %id, error = %e, "failed to create circle claim events");
    }

    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip(state))]
async fn unclaim_item(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.items.unclaim_item(id, auth_user.user_id).await?;

    // Best-effort circle events — don't fail the unclaim if this errors
    if let Err(e) = state.circles.on_item_unclaimed(id, auth_user.user_id).await {
        tracing::warn!(item_id = %id, error = %e, "failed to create circle unclaim events");
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Owner removes a web claim from their own item.
#[tracing::instrument(skip(state))]
async fn owner_unclaim_web(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state
        .items
        .owner_unclaim_web_item(id, auth_user.user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip(state, req))]
async fn batch_delete(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<BatchDeleteRequest>,
) -> Result<StatusCode, AppError> {
    validate_request(&req)?;

    if req.ids.is_empty() {
        return Err(AppError::BadRequest("ids must not be empty".into()));
    }
    if req.ids.len() > 100 {
        return Err(AppError::BadRequest(
            "cannot delete more than 100 items at once".into(),
        ));
    }

    state
        .items
        .batch_delete_items(&req.ids, auth_user.user_id)
        .await?;

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
            image_url: None,
            links: None,
            is_private: None,
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
            image_url: None,
            links: None,
            is_private: None,
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn create_with_links_validates() {
        let mut req = make_create("test");
        req.links = Some(vec![
            "https://example.com".into(),
            "https://other.com".into(),
        ]);
        assert!(req.validate().is_ok());
        assert!(req.validate_links().is_ok());
    }

    #[test]
    fn create_rejects_11_links() {
        let mut req = make_create("test");
        req.links = Some(vec!["https://x.com".into(); 11]);
        assert!(req.validate_links().is_err());
    }

    #[test]
    fn create_resolved_links_from_url() {
        let mut req = make_create("test");
        req.url = Some("https://example.com".into());
        let resolved = req.resolved_links();
        assert_eq!(resolved, Some(vec!["https://example.com".to_string()]));
    }

    #[test]
    fn create_resolved_links_prefers_links_over_url() {
        let mut req = make_create("test");
        req.url = Some("https://old.com".into());
        req.links = Some(vec!["https://new.com".into()]);
        let resolved = req.resolved_links();
        assert_eq!(resolved, Some(vec!["https://new.com".to_string()]));
    }

    // ── UpdateItemRequest ──────────────────────────────────────────

    fn make_update() -> UpdateItemRequest {
        UpdateItemRequest {
            name: None,
            description: None,
            url: None,
            estimated_price: None,
            priority: None,
            category_id: None,
            status: None,
            image_url: None,
            links: None,
            is_private: None,
        }
    }

    #[test]
    fn update_accepts_empty_body() {
        assert!(make_update().validate().is_ok());
    }

    #[test]
    fn update_rejects_empty_name() {
        let mut req = make_update();
        req.name = Some("".into());
        assert!(req.validate().is_err());
    }

    #[test]
    fn update_rejects_256_char_name() {
        let mut req = make_update();
        req.name = Some("a".repeat(256));
        assert!(req.validate().is_err());
    }

    #[test]
    fn update_accepts_valid_partial() {
        let mut req = make_update();
        req.name = Some("renamed".into());
        assert!(req.validate().is_ok());
    }
}
