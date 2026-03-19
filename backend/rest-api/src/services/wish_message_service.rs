use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::dto::pagination::PaginatedResponse;
use crate::dto::wish_messages::MessageResponse;
use crate::errors::AppError;
use crate::models::community_wish::WishStatus;
use crate::traits::{self, NotificationRequest};

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgWishMessageService {
    wish_repo: Arc<dyn traits::CommunityWishRepo>,
    message_repo: Arc<dyn traits::WishMessageRepo>,
    user_repo: Arc<dyn traits::UserRepo>,
    push_token_repo: Arc<dyn traits::PushTokenRepo>,
    notification_svc: Arc<dyn traits::NotificationService>,
}

impl PgWishMessageService {
    pub fn new(
        wish_repo: Arc<dyn traits::CommunityWishRepo>,
        message_repo: Arc<dyn traits::WishMessageRepo>,
        user_repo: Arc<dyn traits::UserRepo>,
        push_token_repo: Arc<dyn traits::PushTokenRepo>,
        notification_svc: Arc<dyn traits::NotificationService>,
    ) -> Self {
        Self {
            wish_repo,
            message_repo,
            user_repo,
            push_token_repo,
            notification_svc,
        }
    }

    fn notify_user(&self, user_id: Uuid, title: String, body: String) {
        let push_token_repo = self.push_token_repo.clone();
        let notification_svc = self.notification_svc.clone();

        tokio::spawn(async move {
            let tokens = match push_token_repo.find_by_user(user_id).await {
                Ok(t) => t,
                Err(e) => {
                    tracing::warn!(%user_id, error = %e, "failed to fetch push tokens for message notification");
                    return;
                }
            };

            let requests: Vec<NotificationRequest> = tokens
                .into_iter()
                .map(|pt| NotificationRequest {
                    device_token: pt.token,
                    title: title.clone(),
                    body: body.clone(),
                    custom_data: std::collections::HashMap::new(),
                    ..Default::default()
                })
                .collect();

            if !requests.is_empty() {
                notification_svc.send_batch(&requests).await;
            }
        });
    }
}

#[async_trait]
impl traits::WishMessageService for PgWishMessageService {
    #[tracing::instrument(skip(self, body))]
    async fn send_message(
        &self,
        wish_id: Uuid,
        sender_id: Uuid,
        body: &str,
    ) -> Result<MessageResponse, AppError> {
        let wish = self
            .wish_repo
            .find_by_id(wish_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("wish not found".into()))?;

        // Only allow sending messages when matched
        let status = WishStatus::parse(&wish.status)
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("invalid wish status")))?;
        if status != WishStatus::Matched {
            return Err(AppError::BadRequest(
                "can only send messages when wish is matched".into(),
            ));
        }

        let is_owner = wish.owner_id == sender_id;
        let is_donor = wish.matched_with == Some(sender_id);
        if !is_owner && !is_donor {
            return Err(AppError::Forbidden(
                "only participants can send messages".into(),
            ));
        }

        let msg = self
            .message_repo
            .create(wish_id, sender_id, body)
            .await
            .map_err(AppError::Internal)?;

        let sender = self
            .user_repo
            .find_by_id(sender_id)
            .await
            .map_err(AppError::Internal)?;
        let sender_name = sender
            .and_then(|u| u.display_name)
            .unwrap_or_else(|| "Quelqu'un".to_string());

        // Notify the other participant
        let other_id = if is_owner {
            wish.matched_with
        } else {
            Some(wish.owner_id)
        };
        if let Some(recipient_id) = other_id {
            self.notify_user(
                recipient_id,
                "Nouveau message".to_string(),
                "Nouveau message sur votre demande d'entraide".to_string(),
            );
        }

        Ok(MessageResponse {
            id: msg.id,
            sender_display_name: sender_name,
            is_mine: true,
            body: msg.body,
            created_at: msg.created_at,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn list_messages(
        &self,
        wish_id: Uuid,
        user_id: Uuid,
        page: i64,
        limit: i64,
        offset: i64,
    ) -> Result<PaginatedResponse<MessageResponse>, AppError> {
        let wish = self
            .wish_repo
            .find_by_id(wish_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("wish not found".into()))?;

        // Allow reading messages if matched, fulfilled, or closed (for history)
        let status = WishStatus::parse(&wish.status)
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("invalid wish status")))?;
        if !matches!(
            status,
            WishStatus::Matched | WishStatus::Fulfilled | WishStatus::Closed
        ) {
            return Err(AppError::BadRequest(
                "messages are only available for matched, fulfilled, or closed wishes".into(),
            ));
        }

        let is_owner = wish.owner_id == user_id;
        let is_donor = wish.matched_with == Some(user_id);
        if !is_owner && !is_donor {
            return Err(AppError::Forbidden(
                "only participants can read messages".into(),
            ));
        }

        let messages = self
            .message_repo
            .list_by_wish(wish_id, limit, offset)
            .await
            .map_err(AppError::Internal)?;
        let total = self
            .message_repo
            .count_by_wish(wish_id)
            .await
            .map_err(AppError::Internal)?;

        // Batch-fetch sender names (dedup to reduce query size)
        let sender_ids: Vec<Uuid> = messages
            .iter()
            .filter_map(|m| m.sender_id)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        let users = self
            .user_repo
            .find_by_ids(&sender_ids)
            .await
            .map_err(AppError::Internal)?;
        let user_map: std::collections::HashMap<Uuid, String> = users
            .into_iter()
            .map(|u| {
                (
                    u.id,
                    u.display_name.unwrap_or_else(|| "Quelqu'un".to_string()),
                )
            })
            .collect();

        let responses: Vec<MessageResponse> = messages
            .into_iter()
            .map(|m| {
                let sender_name = m
                    .sender_id
                    .and_then(|sid| user_map.get(&sid).cloned())
                    .unwrap_or_else(|| "Utilisateur supprimé".to_string());
                MessageResponse {
                    id: m.id,
                    sender_display_name: sender_name,
                    is_mine: m.sender_id == Some(user_id),
                    body: m.body,
                    created_at: m.created_at,
                }
            })
            .collect();

        Ok(PaginatedResponse::new(responses, total, page, limit))
    }
}
