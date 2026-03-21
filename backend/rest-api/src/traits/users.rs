use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::dto::users::{UserDataExport, UserProfileResponse};
use crate::errors::AppError;
use crate::models::User;

#[async_trait]
pub trait UserRepo: Send + Sync {
    async fn create_user(
        &self,
        email: &str,
        username: &str,
        password_hash: &str,
        display_name: Option<&str>,
    ) -> Result<User>;

    async fn find_by_email(&self, email: &str) -> Result<Option<User>>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>>;

    async fn find_by_ids(&self, ids: &[Uuid]) -> Result<Vec<User>>;

    async fn find_by_username(&self, username: &str) -> Result<Option<User>>;

    async fn is_username_taken(
        &self,
        username: &str,
        exclude_user_id: Option<Uuid>,
    ) -> Result<bool>;

    async fn update_profile(
        &self,
        id: Uuid,
        display_name: Option<&str>,
        username: Option<&str>,
        avatar_url: Option<Option<&str>>,
    ) -> Result<Option<User>>;

    async fn delete_user(&self, id: Uuid) -> Result<bool>;

    async fn update_password_hash(&self, id: Uuid, password_hash: &str) -> Result<bool>;

    async fn increment_token_version(&self, id: Uuid) -> Result<i32>;

    async fn get_user_created_at(&self, user_id: Uuid) -> Result<Option<DateTime<Utc>>>;

    async fn create_oauth_user(
        &self,
        email: &str,
        username: &str,
        display_name: Option<&str>,
        oauth_provider: &str,
        oauth_provider_id: &str,
        avatar_url: Option<&str>,
    ) -> Result<User>;

    async fn find_by_oauth(&self, provider: &str, provider_id: &str) -> Result<Option<User>>;

    async fn link_oauth(&self, user_id: Uuid, provider: &str, provider_id: &str) -> Result<bool>;

    /// Link an OAuth provider to an existing user, setting email_verified = true
    /// and backfilling avatar_url / display_name if the user doesn't have them.
    /// Returns the updated User.
    async fn link_oauth_provider(
        &self,
        user_id: Uuid,
        provider: &str,
        provider_id: &str,
        avatar_url: Option<&str>,
        display_name: Option<&str>,
    ) -> Result<User>;

    /// Bulk-fetch display names (display_name ?? username) for a list of user IDs.
    async fn find_display_names(&self, ids: &[Uuid]) -> Result<HashMap<Uuid, String>>;
}

#[async_trait]
pub trait UserService: Send + Sync {
    async fn get_profile(&self, user_id: Uuid) -> Result<UserProfileResponse, AppError>;

    async fn update_profile(
        &self,
        user_id: Uuid,
        req: &crate::dto::users::UpdateProfileRequest,
    ) -> Result<UserProfileResponse, AppError>;

    async fn delete_account(&self, user_id: Uuid) -> Result<(), AppError>;

    async fn export_data(&self, user_id: Uuid) -> Result<UserDataExport, AppError>;

    /// Bulk-fetch display names (display_name ?? username) for a list of user IDs.
    async fn find_display_names(&self, ids: &[Uuid]) -> Result<HashMap<Uuid, String>, AppError>;
}
