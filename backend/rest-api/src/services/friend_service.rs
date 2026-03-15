use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::dto::friends::{
    FriendRequestResponse, FriendResponse, SentFriendRequestResponse, UserSearchResult,
};
use crate::errors::AppError;
use crate::models::friend::FriendRequestStatus;
use crate::repositories::friend_repo;
use crate::traits::{self, NotificationRequest};

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgFriendService {
    pool: sqlx::PgPool,
    friend_repo: Arc<dyn traits::FriendRepo>,
    user_repo: Arc<dyn traits::UserRepo>,
    push_token_repo: Arc<dyn traits::PushTokenRepo>,
    notification_svc: Arc<dyn traits::NotificationService>,
}

impl PgFriendService {
    pub fn new(
        pool: sqlx::PgPool,
        friend_repo: Arc<dyn traits::FriendRepo>,
        user_repo: Arc<dyn traits::UserRepo>,
        push_token_repo: Arc<dyn traits::PushTokenRepo>,
        notification_svc: Arc<dyn traits::NotificationService>,
    ) -> Self {
        Self {
            pool,
            friend_repo,
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
                    tracing::warn!(%user_id, error = %e, "failed to fetch push tokens for friend notification");
                    return;
                }
            };

            let requests: Vec<NotificationRequest> = tokens
                .into_iter()
                .map(|pt| NotificationRequest {
                    device_token: pt.token,
                    title: title.clone(),
                    body: body.clone(),
                })
                .collect();

            if !requests.is_empty() {
                notification_svc.send_batch(&requests).await;
            }
        });
    }
}

#[async_trait]
impl traits::FriendService for PgFriendService {
    #[tracing::instrument(skip(self))]
    async fn search_users(
        &self,
        query: &str,
        requester_id: Uuid,
    ) -> Result<Vec<UserSearchResult>, AppError> {
        let query = query.trim();
        if query.is_empty() || query.len() > 50 {
            return Ok(vec![]);
        }

        let pattern = format!("{query}%");
        let rows: Vec<(String, Option<String>)> = sqlx::query_as(
            "SELECT username, display_name FROM users \
             WHERE username ILIKE $1 AND id != $2 \
             ORDER BY username \
             LIMIT 10",
        )
        .bind(&pattern)
        .bind(requester_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        Ok(rows
            .into_iter()
            .map(|(username, display_name)| UserSearchResult {
                username,
                display_name,
            })
            .collect())
    }

    #[tracing::instrument(skip(self))]
    async fn send_request(
        &self,
        from_user_id: Uuid,
        to_username: &str,
    ) -> Result<FriendRequestResponse, AppError> {
        // Resolve target user (outside tx — read-only, no race concern)
        let target = self
            .user_repo
            .find_by_username(to_username)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("user not found".into()))?;

        if target.id == from_user_id {
            return Err(AppError::BadRequest(
                "cannot send a friend request to yourself".into(),
            ));
        }

        // All checks + mutations in a single transaction with row-level locking
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        // Lock existing friend_requests between these two users (both directions)
        let existing: Vec<crate::models::FriendRequest> = sqlx::query_as(
            "SELECT id, from_user_id, to_user_id, status, created_at \
             FROM friend_requests \
             WHERE (from_user_id = $1 AND to_user_id = $2) \
                OR (from_user_id = $2 AND to_user_id = $1) \
             FOR UPDATE",
        )
        .bind(from_user_id)
        .bind(target.id)
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        // Check for pending requests in either direction
        for req in &existing {
            if req.status == "pending" {
                if req.from_user_id == from_user_id {
                    return Err(AppError::Conflict("friend request already pending".into()));
                } else {
                    return Err(AppError::Conflict(
                        "this user has already sent you a friend request".into(),
                    ));
                }
            }
        }

        // Check already friends (inside tx)
        let already_friends = friend_repo::are_friends(&mut *tx, from_user_id, target.id)
            .await
            .map_err(AppError::Internal)?;

        if already_friends {
            return Err(AppError::Conflict("already friends".into()));
        }

        // Clean up old non-pending requests so re-requesting works
        sqlx::query(
            "DELETE FROM friend_requests \
             WHERE ((from_user_id = $1 AND to_user_id = $2) \
                OR (from_user_id = $2 AND to_user_id = $1)) \
             AND status != 'pending'",
        )
        .bind(from_user_id)
        .bind(target.id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        let req = friend_repo::create_friend_request(&mut *tx, from_user_id, target.id)
            .await
            .map_err(AppError::Internal)?;

        tx.commit()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        // Look up sender info for the response
        let sender = self
            .user_repo
            .find_by_id(from_user_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("sender not found")))?;

        // Notify the target user
        self.notify_user(
            target.id,
            "Demande d'ami".to_string(),
            format!("{} veut vous ajouter en ami", sender.username),
        );

        Ok(FriendRequestResponse {
            id: req.id,
            from_user_id: req.from_user_id,
            from_username: sender.username,
            from_display_name: sender.display_name,
            status: req.status,
            created_at: req.created_at,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn list_pending_requests(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<FriendRequestResponse>, AppError> {
        let reqs = self
            .friend_repo
            .find_pending_requests(user_id)
            .await
            .map_err(AppError::Internal)?;

        if reqs.is_empty() {
            return Ok(vec![]);
        }

        let sender_ids: Vec<Uuid> = reqs.iter().map(|r| r.from_user_id).collect();
        let users = self
            .user_repo
            .find_by_ids(&sender_ids)
            .await
            .map_err(AppError::Internal)?;

        let user_map: std::collections::HashMap<Uuid, _> = users
            .into_iter()
            .map(|u| (u.id, (u.username, u.display_name)))
            .collect();

        let responses = reqs
            .into_iter()
            .map(|r| {
                let (username, display_name) = user_map
                    .get(&r.from_user_id)
                    .cloned()
                    .unwrap_or_else(|| ("unknown".to_string(), None));
                FriendRequestResponse {
                    id: r.id,
                    from_user_id: r.from_user_id,
                    from_username: username,
                    from_display_name: display_name,
                    status: r.status,
                    created_at: r.created_at,
                }
            })
            .collect();

        Ok(responses)
    }

    #[tracing::instrument(skip(self))]
    async fn accept_request(
        &self,
        request_id: Uuid,
        user_id: Uuid,
    ) -> Result<FriendResponse, AppError> {
        let req = self
            .friend_repo
            .find_request_by_id(request_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("friend request not found".into()))?;

        // Verify this user is the recipient
        if req.to_user_id != user_id {
            return Err(AppError::Forbidden(
                "only the recipient can accept a friend request".into(),
            ));
        }

        if req.status != "pending" {
            return Err(AppError::BadRequest(
                "friend request is no longer pending".into(),
            ));
        }

        // Transaction: update status + create friendship
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        let updated =
            friend_repo::update_request_status(&mut *tx, request_id, FriendRequestStatus::Accepted)
                .await
                .map_err(AppError::Internal)?;

        if !updated {
            return Err(AppError::Conflict(
                "friend request was already handled".into(),
            ));
        }

        let friendship = friend_repo::create_friendship(&mut *tx, req.from_user_id, user_id)
            .await
            .map_err(AppError::Internal)?;

        tx.commit()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        // Fetch the sender's info for the response
        let friend_user = self
            .user_repo
            .find_by_id(req.from_user_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("friend user not found")))?;

        // Notify the sender that request was accepted
        if let Some(acceptor) = self
            .user_repo
            .find_by_id(user_id)
            .await
            .map_err(AppError::Internal)?
        {
            self.notify_user(
                req.from_user_id,
                "Demande acceptée !".to_string(),
                format!("{} a accepté votre demande", acceptor.username),
            );
        }

        Ok(FriendResponse {
            user_id: friend_user.id,
            username: friend_user.username,
            display_name: friend_user.display_name,
            since: friendship.created_at,
            shared_item_count: 0,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn decline_request(&self, request_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let req = self
            .friend_repo
            .find_request_by_id(request_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("friend request not found".into()))?;

        if req.to_user_id != user_id {
            return Err(AppError::Forbidden(
                "only the recipient can decline a friend request".into(),
            ));
        }

        if req.status != "pending" {
            return Err(AppError::BadRequest(
                "friend request is no longer pending".into(),
            ));
        }

        self.friend_repo
            .update_request_status(request_id, FriendRequestStatus::Declined)
            .await
            .map_err(AppError::Internal)?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn list_friends(&self, user_id: Uuid) -> Result<Vec<FriendResponse>, AppError> {
        let friends = self
            .friend_repo
            .list_friends_with_since(user_id)
            .await
            .map_err(AppError::Internal)?;

        // Count active items per friend (shared via common circles)
        let friend_ids: Vec<Uuid> = friends.iter().map(|f| f.user_id).collect();
        let item_counts = self
            .friend_repo
            .count_active_items_per_user(&friend_ids)
            .await
            .unwrap_or_default();

        Ok(friends
            .into_iter()
            .map(|f| {
                let count = item_counts.get(&f.user_id).copied().unwrap_or(0);
                FriendResponse {
                    user_id: f.user_id,
                    username: f.username,
                    display_name: f.display_name,
                    since: f.since,
                    shared_item_count: count,
                }
            })
            .collect())
    }

    #[tracing::instrument(skip(self))]
    async fn list_sent_requests(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<SentFriendRequestResponse>, AppError> {
        let reqs = self
            .friend_repo
            .find_sent_requests(user_id)
            .await
            .map_err(AppError::Internal)?;

        if reqs.is_empty() {
            return Ok(vec![]);
        }

        let target_ids: Vec<Uuid> = reqs.iter().map(|r| r.to_user_id).collect();
        let users = self
            .user_repo
            .find_by_ids(&target_ids)
            .await
            .map_err(AppError::Internal)?;

        let user_map: std::collections::HashMap<Uuid, _> = users
            .into_iter()
            .map(|u| (u.id, (u.username, u.display_name)))
            .collect();

        let responses = reqs
            .into_iter()
            .map(|r| {
                let (username, display_name) = user_map
                    .get(&r.to_user_id)
                    .cloned()
                    .unwrap_or_else(|| ("unknown".to_string(), None));
                SentFriendRequestResponse {
                    id: r.id,
                    to_user_id: r.to_user_id,
                    to_username: username,
                    to_display_name: display_name,
                    status: r.status,
                    created_at: r.created_at,
                }
            })
            .collect();

        Ok(responses)
    }

    #[tracing::instrument(skip(self))]
    async fn cancel_request(&self, request_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let req = self
            .friend_repo
            .find_request_by_id(request_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("friend request not found".into()))?;

        if req.from_user_id != user_id {
            return Err(AppError::Forbidden(
                "only the sender can cancel a friend request".into(),
            ));
        }

        if req.status != "pending" {
            return Err(AppError::BadRequest(
                "friend request is no longer pending".into(),
            ));
        }

        self.friend_repo
            .update_request_status(request_id, FriendRequestStatus::Cancelled)
            .await
            .map_err(AppError::Internal)?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn remove_friend(&self, user_id: Uuid, friend_id: Uuid) -> Result<(), AppError> {
        let removed = self
            .friend_repo
            .delete_friendship(user_id, friend_id)
            .await
            .map_err(AppError::Internal)?;

        if !removed {
            return Err(AppError::NotFound("friendship not found".into()));
        }

        // Clean up any friend_requests between the two users
        sqlx::query(
            "DELETE FROM friend_requests \
             WHERE (from_user_id = $1 AND to_user_id = $2) \
                OR (from_user_id = $2 AND to_user_id = $1)",
        )
        .bind(user_id)
        .bind(friend_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        Ok(())
    }
}
