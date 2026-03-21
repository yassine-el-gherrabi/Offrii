use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::dto::categories::CategoryResponse;
use crate::dto::items::ItemResponse;
use crate::dto::users::{UpdateProfileRequest, UserDataExport, UserProfileResponse};
use crate::errors::AppError;
use crate::traits;
use crate::utils::username::{is_reserved_username, is_valid_username};

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgUserService {
    user_repo: Arc<dyn traits::UserRepo>,
    item_repo: Arc<dyn traits::ItemRepo>,
    category_repo: Arc<dyn traits::CategoryRepo>,
    circle_svc: Arc<dyn traits::CircleService>,
    friend_svc: Arc<dyn traits::FriendService>,
    community_wish_svc: Arc<dyn traits::CommunityWishService>,
    wish_message_svc: Arc<dyn traits::WishMessageService>,
}

impl PgUserService {
    pub fn new(
        user_repo: Arc<dyn traits::UserRepo>,
        item_repo: Arc<dyn traits::ItemRepo>,
        category_repo: Arc<dyn traits::CategoryRepo>,
        circle_svc: Arc<dyn traits::CircleService>,
        friend_svc: Arc<dyn traits::FriendService>,
        community_wish_svc: Arc<dyn traits::CommunityWishService>,
        wish_message_svc: Arc<dyn traits::WishMessageService>,
    ) -> Self {
        Self {
            user_repo,
            item_repo,
            category_repo,
            circle_svc,
            friend_svc,
            community_wish_svc,
            wish_message_svc,
        }
    }
}

#[async_trait]
impl traits::UserService for PgUserService {
    async fn get_profile(&self, user_id: Uuid) -> Result<UserProfileResponse, AppError> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("user not found".into()))?;

        Ok(UserProfileResponse::from(&user))
    }

    async fn update_profile(
        &self,
        user_id: Uuid,
        req: &UpdateProfileRequest,
    ) -> Result<UserProfileResponse, AppError> {
        // If nothing to update, just return current profile
        if req.display_name.is_none() && req.username.is_none() && req.avatar_url.is_none() {
            return self.get_profile(user_id).await;
        }

        // Validate username if provided
        if let Some(ref username) = req.username {
            if !is_valid_username(username) {
                return Err(AppError::BadRequest(
                    "username must be 3-30 characters, start with a letter, and contain only lowercase letters, digits, and underscores".into(),
                ));
            }
            if is_reserved_username(username) {
                return Err(AppError::BadRequest("this username is reserved".into()));
            }

            let taken = self
                .user_repo
                .is_username_taken(username, Some(user_id))
                .await
                .map_err(AppError::Internal)?;

            if taken {
                return Err(AppError::Conflict("username already taken".into()));
            }
        }

        let user = self
            .user_repo
            .update_profile(
                user_id,
                req.display_name.as_deref(),
                req.username.as_deref(),
                req.avatar_url.as_ref().map(|v| v.as_deref()),
            )
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("user not found".into()))?;

        Ok(UserProfileResponse::from(&user))
    }

    async fn export_data(&self, user_id: Uuid) -> Result<UserDataExport, AppError> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("user not found".into()))?;

        let items = self
            .item_repo
            .list(user_id, None, None, "created_at", "desc", i64::MAX, 0)
            .await
            .map_err(AppError::Internal)?;

        let categories = self
            .category_repo
            .list_all()
            .await
            .map_err(AppError::Internal)?;

        let circles = self.circle_svc.list_circles(user_id).await?;

        let friends = self.friend_svc.list_friends(user_id).await?;

        let community_wishes = self.community_wish_svc.list_my_wishes(user_id).await?;

        // Collect messages from all user's wishes
        let mut wish_messages = Vec::new();
        for wish in &community_wishes {
            if let Ok(paginated) = self
                .wish_message_svc
                .list_messages(wish.id, user_id, 1, 1000, 0)
                .await
            {
                wish_messages.extend(paginated.data);
            }
        }

        Ok(UserDataExport {
            profile: UserProfileResponse::from(&user),
            items: items.into_iter().map(ItemResponse::from).collect(),
            categories: categories.into_iter().map(CategoryResponse::from).collect(),
            circles,
            friends,
            community_wishes,
            wish_messages,
            exported_at: chrono::Utc::now(),
        })
    }

    async fn delete_account(&self, user_id: Uuid) -> Result<(), AppError> {
        let deleted = self
            .user_repo
            .delete_user(user_id)
            .await
            .map_err(AppError::Internal)?;

        if !deleted {
            return Err(AppError::NotFound("user not found".into()));
        }

        Ok(())
    }
}
