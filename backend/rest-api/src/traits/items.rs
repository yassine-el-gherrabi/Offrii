use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::dto::items::{ItemResponse, ListItemsQuery};
use crate::dto::pagination::PaginatedResponse;
use crate::errors::AppError;
use crate::models::{CircleItem, Item};

#[allow(clippy::too_many_arguments)]
#[async_trait]
pub trait ItemRepo: Send + Sync {
    async fn create(
        &self,
        user_id: Uuid,
        name: &str,
        description: Option<&str>,
        estimated_price: Option<rust_decimal::Decimal>,
        priority: i16,
        category_id: Option<Uuid>,
        image_url: Option<&str>,
        links: Option<&[String]>,
        is_private: bool,
    ) -> Result<Item>;

    async fn find_by_id(&self, id: Uuid, user_id: Uuid) -> Result<Option<Item>>;

    async fn list(
        &self,
        user_id: Uuid,
        status: Option<&str>,
        category_ids: Option<&[Uuid]>,
        sort: &str,
        order: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Item>>;

    async fn count(
        &self,
        user_id: Uuid,
        status: Option<&str>,
        category_ids: Option<&[Uuid]>,
    ) -> Result<i64>;

    async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        estimated_price: Option<rust_decimal::Decimal>,
        priority: Option<i16>,
        category_id: Option<Option<Uuid>>,
        status: Option<&str>,
        image_url: Option<Option<&str>>,
        links: Option<&[String]>,
        is_private: Option<bool>,
    ) -> Result<Option<Item>>;

    async fn soft_delete(&self, id: Uuid, user_id: Uuid) -> Result<bool>;

    async fn find_active_older_than(
        &self,
        user_id: Uuid,
        cutoff: DateTime<Utc>,
    ) -> Result<Vec<Item>>;

    async fn find_by_id_any_user(&self, id: Uuid) -> Result<Option<Item>>;

    async fn claim_item(&self, id: Uuid, claimer_id: Uuid) -> Result<Option<Uuid>>;

    async fn unclaim_item(&self, id: Uuid, claimer_id: Uuid) -> Result<Option<Uuid>>;

    async fn find_by_ids(&self, user_id: Uuid, ids: &[Uuid]) -> Result<Vec<Item>>;

    async fn find_by_ids_any_user(&self, ids: &[Uuid]) -> Result<Vec<Item>>;

    /// Web claim: anonymous user claims an item. Returns (owner_user_id, web_claim_token).
    async fn web_claim_item(
        &self,
        id: Uuid,
        name: &str,
        link_id: Uuid,
    ) -> Result<Option<(Uuid, Uuid)>>;

    /// Web unclaim: anonymous user cancels their claim using the web_claim_token.
    async fn web_unclaim_item(&self, id: Uuid, token: Uuid) -> Result<Option<Uuid>>;

    /// Owner unclaim for web claims: item owner removes a web claim.
    async fn owner_unclaim_web_item(&self, id: Uuid, owner_id: Uuid) -> Result<Option<Uuid>>;

    async fn update_og_metadata(
        &self,
        id: Uuid,
        og_image_url: Option<&str>,
        og_title: Option<&str>,
        og_site_name: Option<&str>,
    ) -> Result<bool>;
}

#[allow(clippy::too_many_arguments)]
#[async_trait]
pub trait ItemService: Send + Sync {
    async fn create_item(
        &self,
        user_id: Uuid,
        name: &str,
        description: Option<&str>,
        estimated_price: Option<rust_decimal::Decimal>,
        priority: Option<i16>,
        category_id: Option<Uuid>,
        image_url: Option<&str>,
        links: Option<&[String]>,
        is_private: Option<bool>,
    ) -> Result<ItemResponse, AppError>;

    async fn get_item(&self, id: Uuid, user_id: Uuid) -> Result<ItemResponse, AppError>;

    async fn list_items(
        &self,
        user_id: Uuid,
        query: &ListItemsQuery,
    ) -> Result<PaginatedResponse<ItemResponse>, AppError>;

    async fn update_item(
        &self,
        id: Uuid,
        user_id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        estimated_price: Option<rust_decimal::Decimal>,
        priority: Option<i16>,
        category_id: Option<Option<Uuid>>,
        status: Option<&str>,
        image_url: Option<Option<&str>>,
        links: Option<&[String]>,
        is_private: Option<bool>,
    ) -> Result<ItemResponse, AppError>;

    async fn delete_item(&self, id: Uuid, user_id: Uuid) -> Result<(), AppError>;

    async fn batch_delete_items(&self, ids: &[Uuid], user_id: Uuid) -> Result<u64, AppError>;

    async fn claim_item(&self, item_id: Uuid, claimer_id: Uuid) -> Result<(), AppError>;

    async fn unclaim_item(&self, item_id: Uuid, claimer_id: Uuid) -> Result<(), AppError>;

    /// Owner unclaim for web claims: item owner can remove a web claim.
    async fn owner_unclaim_web_item(&self, item_id: Uuid, owner_id: Uuid) -> Result<(), AppError>;
}

#[async_trait]
pub trait CircleItemRepo: Send + Sync {
    /// Returns `Some` when a new row was inserted, `None` when item was already shared.
    async fn share_item(
        &self,
        circle_id: Uuid,
        item_id: Uuid,
        shared_by: Uuid,
    ) -> Result<Option<CircleItem>>;

    async fn unshare_item(&self, circle_id: Uuid, item_id: Uuid) -> Result<bool>;

    async fn list_by_circle(&self, circle_id: Uuid) -> Result<Vec<CircleItem>>;

    async fn find(&self, circle_id: Uuid, item_id: Uuid) -> Result<Option<CircleItem>>;

    async fn list_circles_for_item(&self, item_id: Uuid) -> Result<Vec<Uuid>>;

    async fn delete_by_circle_and_user(&self, circle_id: Uuid, user_id: Uuid) -> Result<u64>;

    /// Batch fetch circle names for multiple items.
    /// Returns: item_id -> Vec<(circle_id, name, is_direct)>
    async fn list_circle_names_for_items(
        &self,
        item_ids: &[Uuid],
    ) -> Result<crate::repositories::circle_item_repo::CircleInfoMap>;
}

#[async_trait]
pub trait CategoryRepo: Send + Sync {
    async fn list_all(&self) -> Result<Vec<crate::models::Category>>;
}

#[async_trait]
pub trait CategoryService: Send + Sync {
    async fn list_categories(
        &self,
    ) -> Result<Vec<crate::dto::categories::CategoryResponse>, AppError>;
}
