use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use crate::models::{Notification, PushToken};

#[async_trait]
pub trait NotificationRepo: Send + Sync {
    #[allow(clippy::too_many_arguments)]
    async fn create(
        &self,
        user_id: Uuid,
        notif_type: &str,
        title: &str,
        body: &str,
        circle_id: Option<Uuid>,
        item_id: Option<Uuid>,
        wish_id: Option<Uuid>,
        actor_id: Option<Uuid>,
    ) -> Result<Notification>;

    async fn list_by_user(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Notification>>;

    async fn count_unread(&self, user_id: Uuid) -> Result<i64>;
    async fn count_total(&self, user_id: Uuid) -> Result<i64>;
    async fn mark_read(&self, id: Uuid, user_id: Uuid) -> Result<bool>;
    async fn mark_all_read(&self, user_id: Uuid) -> Result<i64>;
    async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<bool>;
}

#[async_trait]
pub trait PushTokenRepo: Send + Sync {
    async fn upsert(&self, user_id: Uuid, token: &str, platform: &str) -> Result<PushToken>;

    async fn delete_by_token(&self, user_id: Uuid, token: &str) -> Result<bool>;

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<PushToken>>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum NotificationOutcome {
    Sent,
    InvalidToken,
    Error(String),
}

#[derive(Debug, Clone, Default)]
pub struct NotificationRequest {
    pub device_token: String,
    pub title: String,
    pub body: String,
    /// Extra key-value pairs added to the APNs payload under "custom_data".
    pub custom_data: HashMap<String, String>,
    /// APNs localization: key for the body (resolved by iOS against Localizable.strings)
    pub loc_key: Option<String>,
    /// APNs localization: arguments for the loc_key format string
    pub loc_args: Vec<String>,
    /// APNs localization: key for the title
    pub title_loc_key: Option<String>,
}

#[async_trait]
pub trait NotificationService: Send + Sync {
    async fn send_batch(&self, messages: &[NotificationRequest]) -> Vec<NotificationOutcome>;
}

use crate::dto::push_tokens::PushTokenResponse;
use crate::errors::AppError;

#[async_trait]
pub trait PushTokenService: Send + Sync {
    async fn register_token(
        &self,
        user_id: Uuid,
        token: &str,
        platform: &str,
    ) -> Result<PushTokenResponse, AppError>;

    async fn unregister_token(&self, user_id: Uuid, token: &str) -> Result<(), AppError>;
}
