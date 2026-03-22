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
    notification_repo: Arc<dyn traits::NotificationRepo>,
}

impl PgWishMessageService {
    pub fn new(
        wish_repo: Arc<dyn traits::CommunityWishRepo>,
        message_repo: Arc<dyn traits::WishMessageRepo>,
        user_repo: Arc<dyn traits::UserRepo>,
        push_token_repo: Arc<dyn traits::PushTokenRepo>,
        notification_svc: Arc<dyn traits::NotificationService>,
        notification_repo: Arc<dyn traits::NotificationRepo>,
    ) -> Self {
        Self {
            wish_repo,
            message_repo,
            user_repo,
            push_token_repo,
            notification_svc,
            notification_repo,
        }
    }

    fn notify_user(
        &self,
        user_id: Uuid,
        title: String,
        body: String,
        wish_id: Uuid,
        actor_id: Uuid,
        sender_name: String,
    ) {
        let push_token_repo = self.push_token_repo.clone();
        let notification_svc = self.notification_svc.clone();
        let notification_repo = self.notification_repo.clone();

        tokio::spawn(async move {
            // Persist to notification center
            let _ = notification_repo
                .create(
                    user_id,
                    "wish_message",
                    &title,
                    &body,
                    None,           // circle_id
                    None,           // item_id
                    Some(wish_id),  // wish_id
                    Some(actor_id), // actor_id
                )
                .await;

            // Send push notification
            let tokens = match push_token_repo.find_by_user(user_id).await {
                Ok(t) => t,
                Err(e) => {
                    tracing::warn!(%user_id, error = %e, "failed to fetch push tokens for message notification");
                    return;
                }
            };

            // Build custom data for iOS deep link on tap
            let mut custom_data = std::collections::HashMap::new();
            custom_data.insert("type".to_string(), "wish_message".to_string());
            custom_data.insert("wish_id".to_string(), wish_id.to_string());

            // APNs loc-key for client-side localization
            let loc_key = Some("push.wish_message.body".to_string());
            let title_loc_key = Some("push.wish_message.title".to_string());
            let loc_args = vec![sender_name];

            let requests: Vec<NotificationRequest> = tokens
                .into_iter()
                .map(|pt| NotificationRequest {
                    device_token: pt.token,
                    title: title.clone(),
                    body: body.clone(),
                    custom_data: custom_data.clone(),
                    loc_key: loc_key.clone(),
                    loc_args: loc_args.clone(),
                    title_loc_key: title_loc_key.clone(),
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
        let status = wish.status;
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

        // Notify the other participant (fire-and-forget)
        let other_id = if is_owner {
            wish.matched_with
        } else {
            Some(wish.owner_id)
        };
        if let Some(recipient_id) = other_id {
            // Guard: never notify yourself
            if recipient_id != sender_id {
                // Truncate message preview for notification body
                let preview: String = body.chars().take(80).collect();
                let notif_body = format!("{sender_name}: {preview}");

                self.notify_user(
                    recipient_id,
                    "Nouveau message".to_string(),
                    notif_body,
                    wish_id,
                    sender_id,
                    sender_name.clone(),
                );
            }
        }

        metrics::counter!("offrii_wish_messages_sent_total").increment(1);

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
        let status = wish.status;
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

        // Only show messages between current participants (owner + current donor).
        // Prevents a new donor from seeing messages from a previous donor.
        let all_messages = self
            .message_repo
            .list_by_wish(wish_id, limit, offset)
            .await
            .map_err(AppError::Internal)?;

        let mut participants = std::collections::HashSet::new();
        participants.insert(wish.owner_id);
        if let Some(donor_id) = wish.matched_with {
            participants.insert(donor_id);
        }

        let messages: Vec<_> = all_messages
            .into_iter()
            .filter(|m| m.sender_id.is_some_and(|sid| participants.contains(&sid)))
            .collect();
        let total = messages.len() as i64;

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
