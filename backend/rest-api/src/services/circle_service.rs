use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{TimeDelta, Utc};
use rand::distr::Alphanumeric;
use rand::{Rng, rng};
use uuid::Uuid;

use crate::dto::circles::{
    CircleDetailResponse, CircleEventResponse, CircleItemResponse, CircleMemberResponse,
    CircleResponse, ClaimedByInfo, FeedResponse, InviteResponse, JoinResponse,
};
use crate::errors::AppError;
use crate::repositories::{circle_event_repo, circle_invite_repo, circle_member_repo};
use crate::traits::{self, NotificationRequest};

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgCircleService {
    pool: sqlx::PgPool,
    circle_repo: Arc<dyn traits::CircleRepo>,
    circle_member_repo: Arc<dyn traits::CircleMemberRepo>,
    circle_invite_repo: Arc<dyn traits::CircleInviteRepo>,
    circle_item_repo: Arc<dyn traits::CircleItemRepo>,
    circle_event_repo: Arc<dyn traits::CircleEventRepo>,
    item_repo: Arc<dyn traits::ItemRepo>,
    user_repo: Arc<dyn traits::UserRepo>,
    push_token_repo: Arc<dyn traits::PushTokenRepo>,
    notification_svc: Arc<dyn traits::NotificationService>,
    friend_repo: Arc<dyn traits::FriendRepo>,
    redis: redis::Client,
}

#[allow(clippy::too_many_arguments)]
impl PgCircleService {
    pub fn new(
        pool: sqlx::PgPool,
        circle_repo: Arc<dyn traits::CircleRepo>,
        circle_member_repo: Arc<dyn traits::CircleMemberRepo>,
        circle_invite_repo: Arc<dyn traits::CircleInviteRepo>,
        circle_item_repo: Arc<dyn traits::CircleItemRepo>,
        circle_event_repo: Arc<dyn traits::CircleEventRepo>,
        item_repo: Arc<dyn traits::ItemRepo>,
        user_repo: Arc<dyn traits::UserRepo>,
        push_token_repo: Arc<dyn traits::PushTokenRepo>,
        notification_svc: Arc<dyn traits::NotificationService>,
        friend_repo: Arc<dyn traits::FriendRepo>,
        redis: redis::Client,
    ) -> Self {
        Self {
            pool,
            circle_repo,
            circle_member_repo,
            circle_invite_repo,
            circle_item_repo,
            circle_event_repo,
            item_repo,
            user_repo,
            push_token_repo,
            notification_svc,
            friend_repo,
            redis,
        }
    }

    async fn require_membership(&self, circle_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let member = self
            .circle_member_repo
            .find_member(circle_id, user_id)
            .await
            .map_err(AppError::Internal)?;
        if member.is_none() {
            return Err(AppError::Forbidden("not a member of this circle".into()));
        }
        Ok(())
    }

    async fn require_owner(&self, circle_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let circle = self
            .circle_repo
            .find_by_id(circle_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("circle not found".into()))?;
        if circle.owner_id != user_id {
            return Err(AppError::Forbidden(
                "only the circle owner can do this".into(),
            ));
        }
        Ok(())
    }

    fn bump_circle_version(&self, circle_id: Uuid) {
        let redis = self.redis.clone();
        tokio::spawn(async move {
            let ver_key = format!("circle_ver:{circle_id}");
            let Ok(mut conn) = redis.get_multiplexed_async_connection().await else {
                tracing::warn!(%circle_id, "redis unavailable for circle version bump");
                return;
            };
            let _: Result<i64, _> = redis::cmd("INCR")
                .arg(&ver_key)
                .query_async(&mut conn)
                .await;
        });
    }

    fn notify_members(&self, circle_id: Uuid, exclude_user: Uuid, title: String, body: String) {
        let member_repo = self.circle_member_repo.clone();
        let push_token_repo = self.push_token_repo.clone();
        let notification_svc = self.notification_svc.clone();

        tokio::spawn(async move {
            let members = match member_repo.list_members(circle_id).await {
                Ok(m) => m,
                Err(e) => {
                    tracing::warn!(%circle_id, error = %e, "failed to list members for notification");
                    return;
                }
            };

            let mut requests = Vec::new();
            for member in &members {
                if member.user_id == exclude_user {
                    continue;
                }
                let tokens = match push_token_repo.find_by_user(member.user_id).await {
                    Ok(t) => t,
                    Err(_) => continue,
                };
                for pt in tokens {
                    requests.push(NotificationRequest {
                        device_token: pt.token,
                        title: title.clone(),
                        body: body.clone(),
                    });
                }
            }

            if !requests.is_empty() {
                notification_svc.send_batch(&requests).await;
            }
        });
    }

    /// Build a user_id → (username, display_name) lookup from member list.
    async fn user_lookup(
        &self,
        user_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, (String, Option<String>)>, AppError> {
        let users = self
            .user_repo
            .find_by_ids(user_ids)
            .await
            .map_err(AppError::Internal)?;
        Ok(users
            .into_iter()
            .map(|u| (u.id, (u.username, u.display_name)))
            .collect())
    }
}

#[async_trait]
impl traits::CircleService for PgCircleService {
    #[tracing::instrument(skip(self))]
    async fn create_circle(&self, user_id: Uuid, name: &str) -> Result<CircleResponse, AppError> {
        let circle = self
            .circle_repo
            .create(Some(name), user_id, false)
            .await
            .map_err(AppError::Internal)?;

        let count = self
            .circle_member_repo
            .count_members(circle.id)
            .await
            .map_err(AppError::Internal)?;

        Ok(CircleResponse {
            id: circle.id,
            name: circle.name,
            is_direct: circle.is_direct,
            owner_id: circle.owner_id,
            member_count: count,
            created_at: circle.created_at,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn list_circles(&self, user_id: Uuid) -> Result<Vec<CircleResponse>, AppError> {
        let rows = self
            .circle_repo
            .list_by_member(user_id)
            .await
            .map_err(AppError::Internal)?;

        let responses = rows
            .into_iter()
            .map(|(circle, count, other_username)| {
                let name = if circle.is_direct {
                    other_username.or(circle.name)
                } else {
                    circle.name
                };
                CircleResponse {
                    id: circle.id,
                    name,
                    is_direct: circle.is_direct,
                    owner_id: circle.owner_id,
                    member_count: count,
                    created_at: circle.created_at,
                }
            })
            .collect();

        Ok(responses)
    }

    #[tracing::instrument(skip(self))]
    async fn get_circle(
        &self,
        circle_id: Uuid,
        user_id: Uuid,
    ) -> Result<CircleDetailResponse, AppError> {
        self.require_membership(circle_id, user_id).await?;

        let circle = self
            .circle_repo
            .find_by_id(circle_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("circle not found".into()))?;

        let members = self
            .circle_member_repo
            .list_members(circle_id)
            .await
            .map_err(AppError::Internal)?;

        let user_ids: Vec<Uuid> = members.iter().map(|m| m.user_id).collect();
        let user_map = self.user_lookup(&user_ids).await?;

        let member_responses: Vec<CircleMemberResponse> = members
            .into_iter()
            .map(|m| {
                let (username, display_name) = user_map
                    .get(&m.user_id)
                    .cloned()
                    .unwrap_or_else(|| ("unknown".to_string(), None));
                CircleMemberResponse {
                    user_id: m.user_id,
                    username,
                    display_name,
                    role: m.role,
                    joined_at: m.joined_at,
                }
            })
            .collect();

        // For direct circles, compute name as other member's username
        let name = if circle.is_direct {
            member_responses
                .iter()
                .find(|m| m.user_id != user_id)
                .map(|m| m.username.clone())
                .or(circle.name)
        } else {
            circle.name
        };

        Ok(CircleDetailResponse {
            id: circle.id,
            name,
            is_direct: circle.is_direct,
            owner_id: circle.owner_id,
            members: member_responses,
            created_at: circle.created_at,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn update_circle(
        &self,
        circle_id: Uuid,
        user_id: Uuid,
        name: &str,
    ) -> Result<CircleResponse, AppError> {
        self.require_owner(circle_id, user_id).await?;

        let circle = self
            .circle_repo
            .find_by_id(circle_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("circle not found".into()))?;

        if circle.is_direct {
            return Err(AppError::BadRequest(
                "direct circles cannot be renamed".into(),
            ));
        }

        let updated = self
            .circle_repo
            .update_name(circle_id, name)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("circle not found".into()))?;

        let count = self
            .circle_member_repo
            .count_members(circle_id)
            .await
            .map_err(AppError::Internal)?;

        self.bump_circle_version(circle_id);

        Ok(CircleResponse {
            id: updated.id,
            name: updated.name,
            is_direct: updated.is_direct,
            owner_id: updated.owner_id,
            member_count: count,
            created_at: updated.created_at,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn delete_circle(&self, circle_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        self.require_owner(circle_id, user_id).await?;

        self.circle_repo
            .delete(circle_id)
            .await
            .map_err(AppError::Internal)?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn create_direct_circle(
        &self,
        owner_id: Uuid,
        other_user_id: Uuid,
    ) -> Result<CircleResponse, AppError> {
        if owner_id == other_user_id {
            return Err(AppError::BadRequest(
                "cannot create a direct circle with yourself".into(),
            ));
        }

        // Check other user exists
        self.user_repo
            .find_by_id(other_user_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("user not found".into()))?;

        // Check no existing direct circle
        let existing = self
            .circle_member_repo
            .find_direct_circle_between(owner_id, other_user_id)
            .await
            .map_err(AppError::Internal)?;

        if existing.is_some() {
            return Err(AppError::Conflict(
                "direct circle already exists with this user".into(),
            ));
        }

        // Transaction: create circle + add other member
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        let circle = crate::repositories::circle_repo::create(&mut *tx, None, owner_id, true)
            .await
            .map_err(AppError::Internal)?;

        // The trigger adds owner as member. Add the other user.
        crate::repositories::circle_member_repo::add_member(
            &mut *tx,
            circle.id,
            other_user_id,
            "member",
        )
        .await
        .map_err(AppError::Internal)?;

        tx.commit()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        Ok(CircleResponse {
            id: circle.id,
            name: circle.name,
            is_direct: circle.is_direct,
            owner_id: circle.owner_id,
            member_count: 2,
            created_at: circle.created_at,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn create_invite(
        &self,
        circle_id: Uuid,
        user_id: Uuid,
        max_uses: Option<i32>,
        expires_in_hours: Option<i64>,
    ) -> Result<InviteResponse, AppError> {
        self.require_membership(circle_id, user_id).await?;

        let max_uses = max_uses.unwrap_or(1).max(1);
        let hours = expires_in_hours.unwrap_or(24).clamp(1, 720); // 1h to 30d
        let expires_at = Utc::now()
            + TimeDelta::try_hours(hours).unwrap_or_else(|| TimeDelta::try_hours(24).unwrap());

        let token: String = rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let invite = self
            .circle_invite_repo
            .create(circle_id, &token, user_id, expires_at, max_uses)
            .await
            .map_err(AppError::Internal)?;

        Ok(InviteResponse {
            id: invite.id,
            token: invite.token,
            circle_id: invite.circle_id,
            expires_at: invite.expires_at,
            max_uses: invite.max_uses,
            use_count: invite.use_count,
            created_at: invite.created_at,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn join_via_invite(&self, token: &str, user_id: Uuid) -> Result<JoinResponse, AppError> {
        let invite = self
            .circle_invite_repo
            .find_by_token(token)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("invite not found".into()))?;

        // Check expiry
        if invite.expires_at < Utc::now() {
            return Err(AppError::Gone("invite has expired".into()));
        }

        // Check max uses
        if invite.use_count >= invite.max_uses {
            return Err(AppError::Gone("invite has been fully used".into()));
        }

        // Check not already a member
        let existing = self
            .circle_member_repo
            .find_member(invite.circle_id, user_id)
            .await
            .map_err(AppError::Internal)?;

        if existing.is_some() {
            return Err(AppError::Conflict("already a member of this circle".into()));
        }

        // Wrap increment + add_member + event in a single transaction so a
        // failed membership insert cannot consume an invite use.
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        let incremented = circle_invite_repo::increment_use_count(&mut *tx, invite.id)
            .await
            .map_err(AppError::Internal)?;

        if !incremented {
            return Err(AppError::Gone("invite is no longer valid".into()));
        }

        circle_member_repo::add_member(&mut *tx, invite.circle_id, user_id, "member")
            .await
            .map_err(AppError::Internal)?;

        circle_event_repo::insert(
            &mut *tx,
            invite.circle_id,
            user_id,
            "member_joined",
            None,
            None,
        )
        .await
        .map_err(AppError::Internal)?;

        tx.commit()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        // Get circle name for response
        let circle = self
            .circle_repo
            .find_by_id(invite.circle_id)
            .await
            .map_err(AppError::Internal)?;

        // Fire-and-forget notification
        if let Some(user) = self
            .user_repo
            .find_by_id(user_id)
            .await
            .map_err(AppError::Internal)?
        {
            self.notify_members(
                invite.circle_id,
                user_id,
                "Nouveau membre !".to_string(),
                format!("{} a rejoint le cercle", user.username),
            );
        }

        self.bump_circle_version(invite.circle_id);

        Ok(JoinResponse {
            circle_id: invite.circle_id,
            circle_name: circle.and_then(|c| c.name),
        })
    }

    #[tracing::instrument(skip(self))]
    async fn remove_member(
        &self,
        circle_id: Uuid,
        target_user_id: Uuid,
        requester_id: Uuid,
    ) -> Result<(), AppError> {
        self.require_membership(circle_id, requester_id).await?;

        // Self-removal is always allowed; otherwise require owner
        if target_user_id != requester_id {
            self.require_owner(circle_id, requester_id).await?;
        }

        // Cannot remove the owner
        let circle = self
            .circle_repo
            .find_by_id(circle_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("circle not found".into()))?;

        if target_user_id == circle.owner_id && target_user_id != requester_id {
            return Err(AppError::BadRequest(
                "cannot remove the circle owner".into(),
            ));
        }

        let removed = self
            .circle_member_repo
            .remove_member(circle_id, target_user_id)
            .await
            .map_err(AppError::Internal)?;

        if !removed {
            return Err(AppError::NotFound("member not found".into()));
        }

        // Log event
        self.circle_event_repo
            .insert(circle_id, target_user_id, "member_left", None, None)
            .await
            .map_err(AppError::Internal)?;

        self.bump_circle_version(circle_id);

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn list_invites(
        &self,
        circle_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<InviteResponse>, AppError> {
        self.require_membership(circle_id, user_id).await?;

        let invites = self
            .circle_invite_repo
            .list_active_by_circle(circle_id)
            .await
            .map_err(AppError::Internal)?;

        let responses = invites
            .into_iter()
            .map(|inv| InviteResponse {
                id: inv.id,
                token: inv.token,
                circle_id: inv.circle_id,
                expires_at: inv.expires_at,
                max_uses: inv.max_uses,
                use_count: inv.use_count,
                created_at: inv.created_at,
            })
            .collect();

        Ok(responses)
    }

    #[tracing::instrument(skip(self))]
    async fn revoke_invite(
        &self,
        circle_id: Uuid,
        invite_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.require_owner(circle_id, user_id).await?;

        let deleted = self
            .circle_invite_repo
            .delete(invite_id)
            .await
            .map_err(AppError::Internal)?;

        if !deleted {
            return Err(AppError::NotFound("invite not found".into()));
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn share_item(
        &self,
        circle_id: Uuid,
        item_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.require_membership(circle_id, user_id).await?;

        // Verify the user owns the item
        let item = self
            .item_repo
            .find_by_id(item_id, user_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("item not found".into()))?;

        let inserted = self
            .circle_item_repo
            .share_item(circle_id, item_id, user_id)
            .await
            .map_err(AppError::Internal)?;

        // Only emit event / notification / version bump on first share
        if inserted.is_some() {
            self.circle_event_repo
                .insert(circle_id, user_id, "item_shared", Some(item_id), None)
                .await
                .map_err(AppError::Internal)?;

            self.notify_members(
                circle_id,
                user_id,
                "Nouvel article partagé !".to_string(),
                format!("{} a été partagé dans le cercle", item.name),
            );

            self.bump_circle_version(circle_id);
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn list_circle_items(
        &self,
        circle_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<CircleItemResponse>, AppError> {
        self.require_membership(circle_id, user_id).await?;

        let circle_items = self
            .circle_item_repo
            .list_by_circle(circle_id)
            .await
            .map_err(AppError::Internal)?;

        if circle_items.is_empty() {
            return Ok(vec![]);
        }

        let item_ids: Vec<Uuid> = circle_items.iter().map(|ci| ci.item_id).collect();
        let items = self
            .item_repo
            .find_by_ids_any_user(&item_ids)
            .await
            .map_err(AppError::Internal)?;

        let items_map: HashMap<Uuid, _> = items.into_iter().map(|i| (i.id, i)).collect();

        // Build claimer username map for items that are claimed
        let claimer_ids: Vec<Uuid> = items_map.values().filter_map(|i| i.claimed_by).collect();
        let claimer_map = self.user_lookup(&claimer_ids).await?;

        let responses = circle_items
            .into_iter()
            .filter_map(|ci| {
                let item = items_map.get(&ci.item_id)?;
                let is_claimed = item.claimed_by.is_some();

                // Anti-spoiler: item owner can't see who claimed their item
                let claimed_by = if user_id == item.user_id {
                    None
                } else {
                    item.claimed_by.and_then(|cid| {
                        claimer_map.get(&cid).map(|(username, _)| ClaimedByInfo {
                            user_id: cid,
                            username: username.clone(),
                        })
                    })
                };

                Some(CircleItemResponse {
                    id: item.id,
                    name: item.name.clone(),
                    description: item.description.clone(),
                    url: item.url.clone(),
                    estimated_price: item.estimated_price,
                    priority: item.priority,
                    category_id: item.category_id,
                    status: item.status.clone(),
                    is_claimed,
                    claimed_by,
                    shared_at: ci.shared_at,
                    shared_by: ci.shared_by,
                })
            })
            .collect();

        Ok(responses)
    }

    #[tracing::instrument(skip(self))]
    async fn unshare_item(
        &self,
        circle_id: Uuid,
        item_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.require_membership(circle_id, user_id).await?;

        // Check the user shared this item (or is owner of circle)
        let ci = self
            .circle_item_repo
            .find(circle_id, item_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("shared item not found".into()))?;

        if ci.shared_by != user_id {
            self.require_owner(circle_id, user_id).await?;
        }

        self.circle_item_repo
            .unshare_item(circle_id, item_id)
            .await
            .map_err(AppError::Internal)?;

        // Log event
        self.circle_event_repo
            .insert(circle_id, user_id, "item_unshared", Some(item_id), None)
            .await
            .map_err(AppError::Internal)?;

        self.bump_circle_version(circle_id);

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn add_member_by_id(
        &self,
        circle_id: Uuid,
        user_id: Uuid,
        requester_id: Uuid,
    ) -> Result<(), AppError> {
        self.require_membership(circle_id, requester_id).await?;

        // Direct circles are 1-to-1; reject additional members
        let circle = self
            .circle_repo
            .find_by_id(circle_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("circle not found".into()))?;

        if circle.is_direct {
            return Err(AppError::BadRequest(
                "cannot add members to a direct circle".into(),
            ));
        }

        // Verify the target user exists
        let target = self
            .user_repo
            .find_by_id(user_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("user not found".into()))?;

        // Verify friendship
        let friends = self
            .friend_repo
            .are_friends(requester_id, user_id)
            .await
            .map_err(AppError::Internal)?;

        if !friends {
            return Err(AppError::Forbidden(
                "you can only add friends to a circle".into(),
            ));
        }

        // Check not already a member
        let existing = self
            .circle_member_repo
            .find_member(circle_id, user_id)
            .await
            .map_err(AppError::Internal)?;

        if existing.is_some() {
            return Err(AppError::Conflict("user is already a member".into()));
        }

        // Add member + event in transaction
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        circle_member_repo::add_member(&mut *tx, circle_id, user_id, "member")
            .await
            .map_err(AppError::Internal)?;

        circle_event_repo::insert(&mut *tx, circle_id, user_id, "member_joined", None, None)
            .await
            .map_err(AppError::Internal)?;

        tx.commit()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        // Fire-and-forget notification to the added user
        self.notify_members(
            circle_id,
            requester_id,
            "Nouveau membre !".to_string(),
            format!("{} a rejoint le cercle", target.username),
        );

        self.bump_circle_version(circle_id);

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn on_item_claimed(&self, item_id: Uuid, claimer_id: Uuid) -> Result<(), AppError> {
        let circle_ids = self
            .circle_item_repo
            .list_circles_for_item(item_id)
            .await
            .map_err(AppError::Internal)?;

        for circle_id in circle_ids {
            if let Err(e) = self
                .circle_event_repo
                .insert(circle_id, claimer_id, "item_claimed", Some(item_id), None)
                .await
            {
                tracing::warn!(%circle_id, error = %e, "failed to insert item_claimed event");
                continue;
            }
            self.bump_circle_version(circle_id);
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn on_item_unclaimed(&self, item_id: Uuid, claimer_id: Uuid) -> Result<(), AppError> {
        let circle_ids = self
            .circle_item_repo
            .list_circles_for_item(item_id)
            .await
            .map_err(AppError::Internal)?;

        for circle_id in circle_ids {
            if let Err(e) = self
                .circle_event_repo
                .insert(circle_id, claimer_id, "item_unclaimed", Some(item_id), None)
                .await
            {
                tracing::warn!(%circle_id, error = %e, "failed to insert item_unclaimed event");
                continue;
            }
            self.bump_circle_version(circle_id);
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn get_feed(
        &self,
        circle_id: Uuid,
        user_id: Uuid,
        page: Option<i64>,
        per_page: Option<i64>,
    ) -> Result<FeedResponse, AppError> {
        self.require_membership(circle_id, user_id).await?;

        let page = page.unwrap_or(1).max(1);
        let per_page = per_page.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * per_page;

        let events = self
            .circle_event_repo
            .list_by_circle(circle_id, per_page, offset)
            .await
            .map_err(AppError::Internal)?;

        let total = self
            .circle_event_repo
            .count_by_circle(circle_id)
            .await
            .map_err(AppError::Internal)?;

        // Collect all user IDs and item IDs for lookup
        let mut user_ids: Vec<Uuid> = events.iter().map(|e| e.actor_id).collect();
        let target_user_ids: Vec<Uuid> = events.iter().filter_map(|e| e.target_user_id).collect();
        user_ids.extend(&target_user_ids);
        user_ids.sort_unstable();
        user_ids.dedup();

        let user_map = self.user_lookup(&user_ids).await?;

        let item_ids: Vec<Uuid> = events.iter().filter_map(|e| e.target_item_id).collect();
        let items = if item_ids.is_empty() {
            vec![]
        } else {
            self.item_repo
                .find_by_ids_any_user(&item_ids)
                .await
                .map_err(AppError::Internal)?
        };
        let item_map: HashMap<Uuid, _> = items.into_iter().map(|i| (i.id, i)).collect();

        let event_responses = events
            .into_iter()
            .map(|e| {
                // Anti-spoiler: for item_claimed events, hide actor from item owner
                let is_claim_event = e.event_type == "item_claimed";
                let viewer_is_item_owner = e
                    .target_item_id
                    .and_then(|iid| item_map.get(&iid))
                    .map(|item| item.user_id == user_id)
                    .unwrap_or(false);

                let (actor_id, actor_username) = if is_claim_event && viewer_is_item_owner {
                    (None, Some("Quelqu'un".to_string()))
                } else {
                    let username = user_map.get(&e.actor_id).map(|(u, _)| u.clone());
                    (Some(e.actor_id), username)
                };

                let target_item_name = e
                    .target_item_id
                    .and_then(|iid| item_map.get(&iid))
                    .map(|i| i.name.clone());

                let target_username = e
                    .target_user_id
                    .and_then(|uid| user_map.get(&uid))
                    .map(|(u, _)| u.clone());

                CircleEventResponse {
                    id: e.id,
                    event_type: e.event_type,
                    actor_id,
                    actor_username,
                    target_item_id: e.target_item_id,
                    target_item_name,
                    target_user_id: e.target_user_id,
                    target_username,
                    created_at: e.created_at,
                }
            })
            .collect();

        Ok(FeedResponse {
            events: event_responses,
            total,
            page,
            per_page,
        })
    }
}
