use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::dto::circles::{
    CircleDetailResponse, CircleItemResponse, CircleResponse, InviteResponse, JoinResponse,
};
use crate::dto::pagination::PaginatedResponse;
use crate::errors::AppError;
use crate::models::circle::CircleMemberRole;
use crate::models::{Circle, CircleInvite, CircleMember, CircleShareRule};

#[async_trait]
pub trait CircleRepo: Send + Sync {
    async fn create(&self, name: Option<&str>, owner_id: Uuid, is_direct: bool) -> Result<Circle>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Circle>>;

    async fn update(
        &self,
        id: Uuid,
        name: &str,
        image_url: Option<Option<&str>>,
    ) -> Result<Option<Circle>>;

    async fn delete(&self, id: Uuid) -> Result<bool>;

    /// Returns enriched circle rows for the list view.
    async fn list_by_member(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<crate::repositories::circle_repo::CircleListRow>>;
}

#[async_trait]
pub trait CircleMemberRepo: Send + Sync {
    async fn add_member(
        &self,
        circle_id: Uuid,
        user_id: Uuid,
        role: CircleMemberRole,
    ) -> Result<CircleMember>;

    async fn remove_member(&self, circle_id: Uuid, user_id: Uuid) -> Result<bool>;

    async fn find_member(&self, circle_id: Uuid, user_id: Uuid) -> Result<Option<CircleMember>>;

    async fn list_members(&self, circle_id: Uuid) -> Result<Vec<CircleMember>>;

    async fn count_members(&self, circle_id: Uuid) -> Result<i64>;

    async fn find_direct_circle_between(&self, user_a: Uuid, user_b: Uuid) -> Result<Option<Uuid>>;

    async fn is_member(&self, circle_id: Uuid, user_id: Uuid) -> Result<bool>;
}

#[async_trait]
pub trait CircleInviteRepo: Send + Sync {
    async fn create(
        &self,
        circle_id: Uuid,
        token: &str,
        created_by: Uuid,
        expires_at: DateTime<Utc>,
        max_uses: i32,
    ) -> Result<CircleInvite>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<CircleInvite>>;
    async fn find_by_token(&self, token: &str) -> Result<Option<CircleInvite>>;

    async fn increment_use_count(&self, id: Uuid) -> Result<bool>;

    async fn list_active_by_circle(&self, circle_id: Uuid) -> Result<Vec<CircleInvite>>;

    async fn delete(&self, id: Uuid) -> Result<bool>;
}

#[async_trait]
pub trait CircleShareRuleRepo: Send + Sync {
    async fn get(&self, circle_id: Uuid, user_id: Uuid) -> Result<Option<CircleShareRule>>;
    async fn upsert(
        &self,
        circle_id: Uuid,
        user_id: Uuid,
        share_mode: &str,
        category_ids: &[Uuid],
    ) -> Result<CircleShareRule>;
    async fn delete(&self, circle_id: Uuid, user_id: Uuid) -> Result<bool>;
    async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<CircleShareRule>>;
}

#[allow(clippy::too_many_arguments)]
#[async_trait]
pub trait CircleService: Send + Sync {
    async fn create_circle(&self, user_id: Uuid, name: &str) -> Result<CircleResponse, AppError>;

    async fn list_circles(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<CircleResponse>, i64), AppError>;

    async fn get_circle(
        &self,
        circle_id: Uuid,
        user_id: Uuid,
    ) -> Result<CircleDetailResponse, AppError>;

    async fn update_circle(
        &self,
        circle_id: Uuid,
        user_id: Uuid,
        name: &str,
        image_url: Option<Option<&str>>,
    ) -> Result<CircleResponse, AppError>;

    async fn delete_circle(&self, circle_id: Uuid, user_id: Uuid) -> Result<(), AppError>;

    async fn create_direct_circle(
        &self,
        owner_id: Uuid,
        other_user_id: Uuid,
    ) -> Result<CircleResponse, AppError>;

    async fn create_invite(
        &self,
        circle_id: Uuid,
        user_id: Uuid,
        max_uses: Option<i32>,
        expires_in_hours: Option<i64>,
    ) -> Result<InviteResponse, AppError>;

    async fn join_via_invite(&self, token: &str, user_id: Uuid) -> Result<JoinResponse, AppError>;

    async fn remove_member(
        &self,
        circle_id: Uuid,
        target_user_id: Uuid,
        requester_id: Uuid,
    ) -> Result<(), AppError>;

    async fn list_invites(
        &self,
        circle_id: Uuid,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<InviteResponse>, i64), AppError>;

    async fn revoke_invite(
        &self,
        circle_id: Uuid,
        invite_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError>;

    async fn share_item(
        &self,
        circle_id: Uuid,
        item_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError>;

    async fn batch_share_items(
        &self,
        circle_id: Uuid,
        item_ids: &[Uuid],
        user_id: Uuid,
    ) -> Result<(), AppError>;

    async fn list_circle_items(
        &self,
        circle_id: Uuid,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<CircleItemResponse>, i64), AppError>;

    async fn get_circle_item(
        &self,
        circle_id: Uuid,
        item_id: Uuid,
        user_id: Uuid,
    ) -> Result<CircleItemResponse, AppError>;

    async fn unshare_item(
        &self,
        circle_id: Uuid,
        item_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError>;

    async fn get_feed(
        &self,
        circle_id: Uuid,
        user_id: Uuid,
        page: i64,
        limit: i64,
        offset: i64,
    ) -> Result<PaginatedResponse<crate::dto::circles::CircleEventResponse>, AppError>;

    async fn add_member_by_id(
        &self,
        circle_id: Uuid,
        user_id: Uuid,
        requester_id: Uuid,
    ) -> Result<(), AppError>;

    async fn on_item_claimed(&self, item_id: Uuid, claimer_id: Uuid) -> Result<(), AppError>;

    async fn on_item_unclaimed(&self, item_id: Uuid, claimer_id: Uuid) -> Result<(), AppError>;

    async fn on_item_received(&self, item_id: Uuid, owner_id: Uuid) -> Result<(), AppError>;

    async fn on_item_unarchived(&self, item_id: Uuid, owner_id: Uuid) -> Result<(), AppError>;

    /// Returns (circle_name, circle_image_url) for the invite page.
    async fn get_invite_circle_info(
        &self,
        token: &str,
    ) -> Result<(String, Option<String>), AppError>;

    async fn transfer_ownership(
        &self,
        circle_id: Uuid,
        new_owner_id: Uuid,
        requester_id: Uuid,
    ) -> Result<(), AppError>;

    async fn list_reservations(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<crate::dto::circles::ReservationResponse>, i64), AppError>;

    /// Delete all circle_items shared by a user in a given circle.
    async fn unshare_all_for_user(&self, circle_id: Uuid, user_id: Uuid) -> Result<(), AppError>;
}
