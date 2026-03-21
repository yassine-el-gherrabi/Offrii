use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{TimeDelta, Utc};
use rand::distr::Alphanumeric;
use rand::{Rng, rng};
use uuid::Uuid;

use crate::dto::circles::{
    CircleDetailResponse, CircleEventResponse, CircleItemResponse, CircleMemberResponse,
    CircleResponse, ClaimedByInfo, InviteResponse, JoinResponse, ReservationResponse,
};
use crate::dto::pagination::PaginatedResponse;
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
    notification_repo: Arc<dyn traits::NotificationRepo>,
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
        notification_repo: Arc<dyn traits::NotificationRepo>,
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
            notification_repo,
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

    fn bump_item_list_version(&self, user_id: Uuid) {
        let redis = self.redis.clone();
        tokio::spawn(async move {
            let ver_key = format!("items:{user_id}:ver");
            let Ok(mut conn) = redis.get_multiplexed_async_connection().await else {
                return;
            };
            let _: Result<i64, _> = redis::cmd("INCR")
                .arg(&ver_key)
                .query_async(&mut conn)
                .await;
        });
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

    #[allow(dead_code)]
    fn notify_members(&self, circle_id: Uuid, exclude_user: Uuid, title: String, body: String) {
        self.notify_members_with_context(
            circle_id,
            exclude_user,
            title,
            body,
            Some("circle_activity"),
            None,
            Some(exclude_user),
            vec![],
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn notify_members_with_context(
        &self,
        circle_id: Uuid,
        exclude_user: Uuid,
        title: String,
        body: String,
        notif_type: Option<&str>,
        item_id: Option<Uuid>,
        actor_id: Option<Uuid>,
        loc_args: Vec<String>,
    ) {
        let member_repo = self.circle_member_repo.clone();
        let push_token_repo = self.push_token_repo.clone();
        let notification_svc = self.notification_svc.clone();
        let notif_repo = self.notification_repo.clone();
        let user_repo = self.user_repo.clone();
        let persist_type = notif_type.map(|s| s.to_string());

        tokio::spawn(async move {
            let members = match member_repo.list_members(circle_id).await {
                Ok(m) => m,
                Err(e) => {
                    tracing::warn!(%circle_id, error = %e, "failed to list members for notification");
                    return;
                }
            };

            // Resolve actor name for loc_args if not provided
            let loc_args = if loc_args.is_empty() {
                if let Some(aid) = actor_id {
                    if let Ok(Some(user)) = user_repo.find_by_id(aid).await {
                        vec![user.display_name.unwrap_or(user.username)]
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
            } else {
                loc_args
            };

            // Build custom payload for deep link on notification tap
            let mut custom_data = std::collections::HashMap::new();
            custom_data.insert("circle_id".to_string(), circle_id.to_string());
            if let Some(ref ntype) = persist_type {
                custom_data.insert("type".to_string(), ntype.clone());
            }
            if let Some(iid) = item_id {
                custom_data.insert("item_id".to_string(), iid.to_string());
            }

            // APNs loc-key for client-side localization
            let loc_key = persist_type.as_deref().map(|t| format!("push.{t}.body"));
            let title_loc_key = persist_type.as_deref().map(|t| format!("push.{t}.title"));

            let mut requests = Vec::new();
            for member in &members {
                if member.user_id == exclude_user {
                    continue;
                }
                // Persist notification
                if let Some(ref ntype) = persist_type {
                    let _ = notif_repo
                        .create(
                            member.user_id,
                            ntype,
                            &title,
                            &body,
                            Some(circle_id),
                            item_id,
                            None, // wish_id
                            actor_id,
                        )
                        .await;
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
                        custom_data: custom_data.clone(),
                        loc_key: loc_key.clone(),
                        loc_args: loc_args.clone(),
                        title_loc_key: title_loc_key.clone(),
                    });
                }
            }

            if !requests.is_empty() {
                notification_svc.send_batch(&requests).await;
            }
        });
    }

    #[allow(clippy::too_many_arguments)]
    fn notify_user_with_context(
        &self,
        user_id: Uuid,
        title: String,
        body: String,
        notif_type: Option<&str>,
        circle_id: Option<Uuid>,
        item_id: Option<Uuid>,
        actor_id: Option<Uuid>,
        loc_args: Vec<String>,
    ) {
        let push_token_repo = self.push_token_repo.clone();
        let notification_svc = self.notification_svc.clone();
        let notif_repo = self.notification_repo.clone();
        let user_repo_clone = self.user_repo.clone();
        let persist_type = notif_type.map(|s| s.to_string());

        tokio::spawn(async move {
            // Persist notification
            if let Some(ref ntype) = persist_type {
                let _ = notif_repo
                    .create(
                        user_id, ntype, &title, &body, circle_id, item_id, None, actor_id,
                    )
                    .await;
            }

            // Resolve actor name for loc_args if not provided
            let loc_args = if loc_args.is_empty() {
                if let Some(aid) = actor_id {
                    if let Ok(Some(user)) = user_repo_clone.find_by_id(aid).await {
                        vec![user.display_name.unwrap_or(user.username)]
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
            } else {
                loc_args
            };

            // Build custom payload for deep link on notification tap
            let mut custom_data = std::collections::HashMap::new();
            if let Some(ref ntype) = persist_type {
                custom_data.insert("type".to_string(), ntype.clone());
            }
            if let Some(cid) = circle_id {
                custom_data.insert("circle_id".to_string(), cid.to_string());
            }
            if let Some(iid) = item_id {
                custom_data.insert("item_id".to_string(), iid.to_string());
            }

            // APNs loc-key for client-side localization
            let loc_key = persist_type.as_deref().map(|t| format!("push.{t}.body"));
            let title_loc_key = persist_type.as_deref().map(|t| format!("push.{t}.title"));

            let tokens = match push_token_repo.find_by_user(user_id).await {
                Ok(t) => t,
                Err(e) => {
                    tracing::warn!(%user_id, error = %e, "failed to fetch push tokens");
                    return;
                }
            };

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

    /// Build a user_id → (username, display_name) lookup from member list.
    async fn owner_lookup(
        &self,
        user_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, (String, Option<String>, Option<String>)>, AppError> {
        let users = self
            .user_repo
            .find_by_ids(user_ids)
            .await
            .map_err(AppError::Internal)?;
        Ok(users
            .into_iter()
            .map(|u| (u.id, (u.username, u.display_name, u.avatar_url)))
            .collect())
    }

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

    /// Resolve items for any circle via share rules (direct + group).
    /// Members with no rule but existing circle_items are treated as implicit selection.
    async fn list_circle_items_via_rules(
        &self,
        circle_id: Uuid,
        viewer_id: Uuid,
    ) -> Result<Vec<CircleItemResponse>, AppError> {
        use sqlx::Row;

        let members = self
            .circle_member_repo
            .list_members(circle_id)
            .await
            .map_err(AppError::Internal)?;

        let mut all_items: Vec<crate::models::Item> = Vec::new();
        let mut item_owner_map: HashMap<Uuid, Uuid> = HashMap::new();

        for member in &members {
            let rule: Option<crate::models::CircleShareRule> = sqlx::query_as(
                "SELECT * FROM circle_share_rules WHERE circle_id = $1 AND user_id = $2",
            )
            .bind(circle_id)
            .bind(member.user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

            let items: Vec<crate::models::Item> = match rule.as_ref().map(|r| r.share_mode.as_str())
            {
                Some("all") => {
                    sqlx::query_as(
                        "SELECT * FROM items WHERE user_id = $1 AND status = 'active' AND is_private = false ORDER BY created_at DESC",
                    )
                    .bind(member.user_id)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| AppError::Internal(e.into()))?
                }
                Some("categories") => {
                    let cat_ids = &rule.as_ref().unwrap().category_ids;
                    if cat_ids.is_empty() {
                        vec![]
                    } else {
                        sqlx::query_as(
                            "SELECT * FROM items WHERE user_id = $1 AND status = 'active' AND is_private = false AND category_id = ANY($2) ORDER BY created_at DESC",
                        )
                        .bind(member.user_id)
                        .bind(cat_ids)
                        .fetch_all(&self.pool)
                        .await
                        .map_err(|e| AppError::Internal(e.into()))?
                    }
                }
                Some("selection") => {
                    let ci: Vec<crate::models::CircleItem> = sqlx::query_as(
                        "SELECT * FROM circle_items WHERE circle_id = $1 AND shared_by = $2",
                    )
                    .bind(circle_id)
                    .bind(member.user_id)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| AppError::Internal(e.into()))?;

                    let ids: Vec<Uuid> = ci.iter().map(|c| c.item_id).collect();
                    if ids.is_empty() {
                        vec![]
                    } else {
                        self.item_repo
                            .find_by_ids_any_user(&ids)
                            .await
                            .map_err(AppError::Internal)?
                            .into_iter()
                            .filter(|i| !i.is_private)
                            .collect()
                    }
                }
                Some("none") => vec![],
                None => {
                    // Backward compat: no rule but existing circle_items → implicit selection
                    let ci: Vec<crate::models::CircleItem> = sqlx::query_as(
                        "SELECT * FROM circle_items WHERE circle_id = $1 AND shared_by = $2",
                    )
                    .bind(circle_id)
                    .bind(member.user_id)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| AppError::Internal(e.into()))?;

                    let ids: Vec<Uuid> = ci.iter().map(|c| c.item_id).collect();
                    if ids.is_empty() {
                        vec![]
                    } else {
                        self.item_repo
                            .find_by_ids_any_user(&ids)
                            .await
                            .map_err(AppError::Internal)?
                            .into_iter()
                            .filter(|i| !i.is_private)
                            .collect()
                    }
                }
                _ => vec![],
            };

            for item in items {
                item_owner_map.insert(item.id, member.user_id);
                all_items.push(item);
            }
        }

        if all_items.is_empty() {
            return Ok(vec![]);
        }

        let claimer_ids: Vec<Uuid> = all_items.iter().filter_map(|i| i.claimed_by).collect();
        let claimer_map = self.user_lookup(&claimer_ids).await?;
        let owner_ids: Vec<Uuid> = item_owner_map.values().copied().collect();
        let owner_info = self.owner_lookup(&owner_ids).await?;

        let cat_ids: Vec<Uuid> = all_items.iter().filter_map(|i| i.category_id).collect();
        let cat_icon_map: HashMap<Uuid, String> = if !cat_ids.is_empty() {
            sqlx::query("SELECT id, icon FROM categories WHERE id = ANY($1)")
                .bind(&cat_ids)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| AppError::Internal(e.into()))?
                .into_iter()
                .map(|row| (row.get("id"), row.get("icon")))
                .collect()
        } else {
            HashMap::new()
        };

        let now = chrono::Utc::now();
        let responses = all_items
            .into_iter()
            .map(|item| {
                let owner_id = item_owner_map
                    .get(&item.id)
                    .copied()
                    .unwrap_or(item.user_id);
                let is_claimed = item.claimed_by.is_some();
                let claimed_by = if viewer_id == item.user_id {
                    None
                } else {
                    item.claimed_by.and_then(|cid| {
                        claimer_map
                            .get(&cid)
                            .map(|(username, display_name)| ClaimedByInfo {
                                user_id: cid,
                                username: username.clone(),
                                display_name: display_name.clone(),
                            })
                    })
                };

                CircleItemResponse {
                    id: item.id,
                    name: item.name,
                    description: item.description,
                    url: item.url,
                    estimated_price: item.estimated_price,
                    priority: item.priority,
                    category_id: item.category_id,
                    category_icon: item
                        .category_id
                        .and_then(|cid| cat_icon_map.get(&cid).cloned()),
                    status: item.status,
                    is_claimed,
                    claimed_by,
                    image_url: item.image_url,
                    links: item.links,
                    og_image_url: item.og_image_url,
                    og_title: item.og_title,
                    og_site_name: item.og_site_name,
                    shared_at: now,
                    shared_by: owner_id,
                    shared_by_name: owner_info.get(&owner_id).and_then(|(_, dn, _)| dn.clone()),
                    shared_by_avatar_url: owner_info
                        .get(&owner_id)
                        .and_then(|(_, _, av)| av.clone()),
                }
            })
            .collect();

        Ok(responses)
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
            image_url: circle.image_url.clone(),
            member_count: count,
            unreserved_item_count: 0,
            last_activity: None,
            last_activity_at: None,
            member_names: vec![],
            member_ids: vec![],
            member_avatars: vec![],
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
            .map(|row| {
                let name = if row.circle.is_direct {
                    row.other_username.or(row.circle.name)
                } else {
                    row.circle.name
                };

                let last_activity = match (
                    row.last_activity_event_type.as_deref(),
                    row.last_activity_actor.as_deref(),
                    row.last_activity_item.as_deref(),
                ) {
                    (Some("item_shared"), Some(actor), Some(item)) => {
                        Some(format!("{actor} a partagé {item}"))
                    }
                    (Some("item_claimed"), Some(actor), Some(item)) => {
                        Some(format!("{actor} a réservé {item}"))
                    }
                    (Some("item_unclaimed"), _, Some(item)) => {
                        Some(format!("{item} n'est plus réservé"))
                    }
                    (Some("member_joined"), Some(actor), _) => {
                        Some(format!("{actor} a rejoint le cercle"))
                    }
                    (Some("member_left"), Some(actor), _) => {
                        Some(format!("{actor} a quitté le cercle"))
                    }
                    (Some("item_unshared"), _, Some(item)) => Some(format!("{item} a été retiré")),
                    _ => None,
                };

                CircleResponse {
                    id: row.circle.id,
                    name,
                    is_direct: row.circle.is_direct,
                    owner_id: row.circle.owner_id,
                    image_url: row.circle.image_url,
                    member_count: row.member_count,
                    unreserved_item_count: row.unreserved_item_count,
                    last_activity,
                    last_activity_at: row.last_activity_at,
                    member_names: row.member_names,
                    member_ids: row.member_ids,
                    member_avatars: row.member_avatars,
                    created_at: row.circle.created_at,
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
        let user_map = self.owner_lookup(&user_ids).await?;

        let member_responses: Vec<CircleMemberResponse> = members
            .into_iter()
            .map(|m| {
                let (username, display_name, avatar_url) = user_map
                    .get(&m.user_id)
                    .cloned()
                    .unwrap_or_else(|| ("unknown".to_string(), None, None));
                CircleMemberResponse {
                    user_id: m.user_id,
                    username,
                    display_name,
                    avatar_url,
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
            image_url: circle.image_url.clone(),
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
        image_url: Option<Option<&str>>,
    ) -> Result<CircleResponse, AppError> {
        self.require_membership(circle_id, user_id).await?;

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
            .update(circle_id, name, image_url)
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
            image_url: updated.image_url,
            member_count: count,
            unreserved_item_count: 0,
            last_activity: None,
            last_activity_at: None,
            member_names: vec![],
            member_ids: vec![],
            member_avatars: vec![],
            created_at: updated.created_at,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn delete_circle(&self, circle_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        self.require_owner(circle_id, user_id).await?;

        // Direct circles are managed via the friend system — cannot be deleted directly
        let circle = self
            .circle_repo
            .find_by_id(circle_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("circle not found".into()))?;

        if circle.is_direct {
            return Err(AppError::BadRequest(
                "direct circles cannot be deleted; remove the friend instead".into(),
            ));
        }

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

        // Require friendship
        let are_friends = self
            .friend_repo
            .are_friends(owner_id, other_user_id)
            .await
            .map_err(AppError::Internal)?;

        if !are_friends {
            return Err(AppError::Forbidden(
                "you must be friends to create a direct circle".into(),
            ));
        }

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
            image_url: circle.image_url.clone(),
            member_count: 2,
            unreserved_item_count: 0,
            last_activity: None,
            last_activity_at: None,
            member_names: vec![],
            member_ids: vec![],
            member_avatars: vec![],
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

        // Look up creator name
        let creator_name = self
            .user_repo
            .find_by_id(invite.created_by)
            .await
            .ok()
            .flatten()
            .and_then(|u| u.display_name.or(Some(u.username)));

        Ok(InviteResponse {
            id: invite.id,
            token: invite.token.clone(),
            url: String::new(),
            circle_id: invite.circle_id,
            created_by: invite.created_by,
            created_by_name: creator_name,
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
            self.notify_members_with_context(
                invite.circle_id,
                user_id,
                "Nouveau membre !".to_string(),
                format!("{} a rejoint le cercle", user.username),
                Some("circle_member_joined"),
                None,
                Some(user_id),
                vec![user.display_name.unwrap_or(user.username)],
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

        // Direct circles are managed via the friend system
        let circle = self
            .circle_repo
            .find_by_id(circle_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("circle not found".into()))?;

        if circle.is_direct {
            return Err(AppError::BadRequest(
                "cannot leave a direct circle; remove the friend instead".into(),
            ));
        }

        // Self-removal is always allowed; otherwise require owner
        if target_user_id != requester_id {
            self.require_owner(circle_id, requester_id).await?;
        }

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

        // Remove items shared by the departing member
        sqlx::query("DELETE FROM circle_items WHERE circle_id = $1 AND shared_by = $2")
            .bind(circle_id)
            .bind(target_user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

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

        let creator_ids: Vec<Uuid> = invites.iter().map(|i| i.created_by).collect();
        let creator_map = self.user_lookup(&creator_ids).await?;

        let responses = invites
            .into_iter()
            .map(|inv| {
                let name = creator_map
                    .get(&inv.created_by)
                    .map(|(username, display_name)| {
                        display_name.clone().unwrap_or_else(|| username.clone())
                    });
                InviteResponse {
                    id: inv.id,
                    token: inv.token.clone(),
                    url: String::new(),
                    circle_id: inv.circle_id,
                    created_by: inv.created_by,
                    created_by_name: name,
                    expires_at: inv.expires_at,
                    max_uses: inv.max_uses,
                    use_count: inv.use_count,
                    created_at: inv.created_at,
                }
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
        // Allow creator of the invite OR circle owner to revoke
        self.require_membership(circle_id, user_id).await?;

        let invite = self
            .circle_invite_repo
            .find_by_id(invite_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("invite not found".into()))?;

        if invite.created_by != user_id {
            self.require_owner(circle_id, user_id).await?;
        }

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

            self.notify_members_with_context(
                circle_id,
                user_id,
                "New wish shared!".to_string(),
                format!("{} was shared", item.name),
                Some("item_shared"),
                Some(item_id),
                Some(user_id),
                vec![], // auto-resolves actor name
            );

            self.bump_circle_version(circle_id);
            self.bump_item_list_version(user_id);
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn batch_share_items(
        &self,
        circle_id: Uuid,
        item_ids: &[Uuid],
        user_id: Uuid,
    ) -> Result<(), AppError> {
        if item_ids.is_empty() {
            return Ok(());
        }

        self.require_membership(circle_id, user_id).await?;

        let mut shared_count = 0u32;

        for &item_id in item_ids {
            // Verify ownership
            let item = self
                .item_repo
                .find_by_id(item_id, user_id)
                .await
                .map_err(AppError::Internal)?;

            let Some(_item) = item else { continue };

            let inserted = self
                .circle_item_repo
                .share_item(circle_id, item_id, user_id)
                .await
                .map_err(AppError::Internal)?;

            if inserted.is_some() {
                let _ = self
                    .circle_event_repo
                    .insert(circle_id, user_id, "item_shared", Some(item_id), None)
                    .await;

                shared_count += 1;
            }
        }

        // Send ONE notification for all items
        if shared_count > 0 {
            let user = self
                .user_repo
                .find_by_id(user_id)
                .await
                .map_err(AppError::Internal)?;
            let username = user
                .as_ref()
                .and_then(|u| u.display_name.clone())
                .unwrap_or_else(|| {
                    user.as_ref()
                        .map(|u| u.username.clone())
                        .unwrap_or_default()
                });

            let body = format!("{username} shared {shared_count} wish(es)");

            self.notify_members_with_context(
                circle_id,
                user_id,
                "New wishes!".to_string(),
                body,
                Some("item_shared"),
                None,
                Some(user_id),
                vec![username],
            );

            self.bump_circle_version(circle_id);
            self.bump_item_list_version(user_id);
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

        // All circles (direct + group) use share rules with backward-compat fallback
        self.list_circle_items_via_rules(circle_id, user_id).await
    }

    #[tracing::instrument(skip(self))]
    async fn get_circle_item(
        &self,
        circle_id: Uuid,
        item_id: Uuid,
        user_id: Uuid,
    ) -> Result<CircleItemResponse, AppError> {
        // Use the same rule-based resolution as list_circle_items to find the item
        let all_items = self.list_circle_items_via_rules(circle_id, user_id).await?;
        let item_response = all_items
            .into_iter()
            .find(|i| i.id == item_id)
            .ok_or_else(|| AppError::NotFound("item not shared in this circle".into()))?;
        Ok(item_response)
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
        self.bump_item_list_version(user_id);

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

        // N6: Notify the added user specifically
        let circle_name = circle.name.as_deref().unwrap_or("a circle");
        self.notify_user_with_context(
            user_id,
            "New circle".to_string(),
            format!("You were added to circle {circle_name}"),
            Some("circle_added"),
            Some(circle_id),
            None,
            Some(requester_id),
            vec![circle_name.to_string()],
        );

        // Notify other members that someone joined
        self.notify_members_with_context(
            circle_id,
            user_id, // exclude the added user (they got their own notif)
            "Nouveau membre !".to_string(),
            format!("{} a rejoint le cercle", target.username),
            Some("circle_member_joined"),
            None,
            Some(user_id),
            vec![
                target
                    .display_name
                    .clone()
                    .unwrap_or_else(|| target.username.clone()),
            ],
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

        for circle_id in &circle_ids {
            if let Err(e) = self
                .circle_event_repo
                .insert(*circle_id, claimer_id, "item_claimed", Some(item_id), None)
                .await
            {
                tracing::warn!(%circle_id, error = %e, "failed to insert item_claimed event");
                continue;
            }
            self.bump_circle_version(*circle_id);
        }

        // N1+N2: Send push notifications
        let item = self
            .item_repo
            .find_by_id_any_user(item_id)
            .await
            .ok()
            .flatten();
        if let Some(ref item) = item {
            let item_name = &item.name;
            let owner_id = item.user_id;

            // N1: Notify owner (don't reveal who claimed)
            self.notify_user_with_context(
                owner_id,
                "Wish reserved".to_string(),
                format!("Your wish '{}' has been reserved", item_name),
                Some("item_claimed"),
                circle_ids.first().copied(),
                Some(item_id),
                Some(claimer_id),
                vec![item_name.clone()],
            );

            // N2: Notify circle members (reveal claimer, exclude owner + claimer)
            let claimer = self.user_repo.find_by_id(claimer_id).await.ok().flatten();
            let claimer_name = claimer
                .as_ref()
                .and_then(|u| u.display_name.clone())
                .unwrap_or_else(|| {
                    claimer
                        .as_ref()
                        .map(|u| u.username.clone())
                        .unwrap_or_default()
                });

            for circle_id in &circle_ids {
                let circle = self.circle_repo.find_by_id(*circle_id).await.ok().flatten();
                let circle_name = circle
                    .as_ref()
                    .and_then(|c| c.name.clone())
                    .unwrap_or_default();

                // Notify all members except owner and claimer
                let members = self
                    .circle_member_repo
                    .list_members(*circle_id)
                    .await
                    .unwrap_or_default();
                for member in &members {
                    if member.user_id == owner_id || member.user_id == claimer_id {
                        continue;
                    }
                    self.notify_user_with_context(
                        member.user_id,
                        "Wish reserved".to_string(),
                        format!(
                            "'{}' reserved by {} in {}",
                            item_name, claimer_name, circle_name
                        ),
                        Some("item_claimed"),
                        Some(*circle_id),
                        Some(item_id),
                        Some(claimer_id),
                        vec![item_name.clone()],
                    );
                }
            }
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

        for circle_id in &circle_ids {
            if let Err(e) = self
                .circle_event_repo
                .insert(
                    *circle_id,
                    claimer_id,
                    "item_unclaimed",
                    Some(item_id),
                    None,
                )
                .await
            {
                tracing::warn!(%circle_id, error = %e, "failed to insert item_unclaimed event");
                continue;
            }
            self.bump_circle_version(*circle_id);
        }

        // N3: Send push notifications
        let item = self
            .item_repo
            .find_by_id_any_user(item_id)
            .await
            .ok()
            .flatten();
        if let Some(ref item) = item {
            let item_name = &item.name;
            let owner_id = item.user_id;

            // Notify owner
            self.notify_user_with_context(
                owner_id,
                "Wish available".to_string(),
                format!("Your wish '{}' is available again", item_name),
                Some("item_unclaimed"),
                circle_ids.first().copied(),
                Some(item_id),
                Some(claimer_id),
                vec![item_name.clone()],
            );

            // Notify circle members (except owner and unclaimer)
            for circle_id in &circle_ids {
                let circle = self.circle_repo.find_by_id(*circle_id).await.ok().flatten();
                let circle_name = circle
                    .as_ref()
                    .and_then(|c| c.name.clone())
                    .unwrap_or_default();

                let members = self
                    .circle_member_repo
                    .list_members(*circle_id)
                    .await
                    .unwrap_or_default();
                for member in &members {
                    if member.user_id == owner_id || member.user_id == claimer_id {
                        continue;
                    }
                    self.notify_user_with_context(
                        member.user_id,
                        "Wish available".to_string(),
                        format!("'{}' is available again in {}", item_name, circle_name),
                        Some("item_unclaimed"),
                        Some(*circle_id),
                        Some(item_id),
                        Some(claimer_id),
                        vec![item_name.clone()],
                    );
                }
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn on_item_received(&self, item_id: Uuid, owner_id: Uuid) -> Result<(), AppError> {
        let circle_ids = self
            .circle_item_repo
            .list_circles_for_item(item_id)
            .await
            .map_err(AppError::Internal)?;

        for circle_id in &circle_ids {
            if let Err(e) = self
                .circle_event_repo
                .insert(*circle_id, owner_id, "item_received", Some(item_id), None)
                .await
            {
                tracing::warn!(%circle_id, error = %e, "failed to insert item_received event");
                continue;
            }
            self.bump_circle_version(*circle_id);
        }

        // Notify the claimer that the owner received their gift
        let item = self
            .item_repo
            .find_by_id_any_user(item_id)
            .await
            .ok()
            .flatten();
        if let Some(ref item) = item
            && let Some(claimer_id) = item.claimed_by
        {
            let owner = self.user_repo.find_by_id(owner_id).await.ok().flatten();
            let owner_name = owner
                .as_ref()
                .and_then(|u| u.display_name.clone())
                .unwrap_or_else(|| {
                    owner
                        .as_ref()
                        .map(|u| u.username.clone())
                        .unwrap_or_default()
                });

            self.notify_user_with_context(
                claimer_id,
                "Gift received!".to_string(),
                format!("{} received {}!", owner_name, item.name),
                Some("item_received"),
                circle_ids.first().copied(),
                Some(item_id),
                Some(owner_id),
                vec![owner_name, item.name.clone()],
            );
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn on_item_unarchived(&self, item_id: Uuid, owner_id: Uuid) -> Result<(), AppError> {
        // Fetch circle_ids first (needed for both notification context and version bump)
        let circle_ids = self
            .circle_item_repo
            .list_circles_for_item(item_id)
            .await
            .map_err(AppError::Internal)?;

        // Notify the claimer that the item is back in the wishlist
        let item = self
            .item_repo
            .find_by_id_any_user(item_id)
            .await
            .ok()
            .flatten();
        if let Some(ref item) = item
            && let Some(claimer_id) = item.claimed_by
        {
            let owner = self.user_repo.find_by_id(owner_id).await.ok().flatten();
            let owner_name = owner
                .as_ref()
                .and_then(|u| u.display_name.clone())
                .unwrap_or_else(|| {
                    owner
                        .as_ref()
                        .map(|u| u.username.clone())
                        .unwrap_or_default()
                });

            self.notify_user_with_context(
                claimer_id,
                "Wish is back".to_string(),
                format!("{} is back in {}'s wishlist", item.name, owner_name),
                Some("item_unarchived"),
                circle_ids.first().copied(),
                Some(item_id),
                Some(owner_id),
                vec![item.name.clone()],
            );
        }

        // Bump circle versions
        for circle_id in &circle_ids {
            self.bump_circle_version(*circle_id);
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn get_feed(
        &self,
        circle_id: Uuid,
        user_id: Uuid,
        page: i64,
        limit: i64,
        offset: i64,
    ) -> Result<PaginatedResponse<CircleEventResponse>, AppError> {
        self.require_membership(circle_id, user_id).await?;

        let events = self
            .circle_event_repo
            .list_by_circle(circle_id, limit, offset)
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

        let event_responses: Vec<CircleEventResponse> = events
            .into_iter()
            .filter_map(|e| {
                let is_claim_event =
                    e.event_type == "item_claimed" || e.event_type == "item_unclaimed";
                let viewer_is_item_owner = e
                    .target_item_id
                    .and_then(|iid| item_map.get(&iid))
                    .map(|item| item.user_id == user_id)
                    .unwrap_or(false);

                // Anti-spoiler: completely hide claim/unclaim events from the item owner
                if is_claim_event && viewer_is_item_owner {
                    return None;
                }

                let username = user_map
                    .get(&e.actor_id)
                    .map(|(u, dn)| dn.as_deref().unwrap_or(u).to_string());

                let target_item_name = e
                    .target_item_id
                    .and_then(|iid| item_map.get(&iid))
                    .map(|i| i.name.clone());

                let target_username = e
                    .target_user_id
                    .and_then(|uid| user_map.get(&uid))
                    .map(|(u, dn)| dn.as_deref().unwrap_or(u).to_string());

                Some(CircleEventResponse {
                    id: e.id,
                    event_type: e.event_type,
                    actor_id: Some(e.actor_id),
                    actor_username: username,
                    target_item_id: e.target_item_id,
                    target_item_name,
                    target_user_id: e.target_user_id,
                    target_username,
                    created_at: e.created_at,
                })
            })
            .collect();

        Ok(PaginatedResponse::new(event_responses, total, page, limit))
    }

    async fn get_invite_circle_info(
        &self,
        token: &str,
    ) -> Result<(String, Option<String>), AppError> {
        let invite = self
            .circle_invite_repo
            .find_by_token(token)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("invite not found".into()))?;

        let circle = self
            .circle_repo
            .find_by_id(invite.circle_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("circle not found".into()))?;

        Ok((circle.name.unwrap_or_default(), circle.image_url))
    }

    #[tracing::instrument(skip(self))]
    async fn transfer_ownership(
        &self,
        circle_id: Uuid,
        new_owner_id: Uuid,
        requester_id: Uuid,
    ) -> Result<(), AppError> {
        self.require_owner(circle_id, requester_id).await?;
        self.require_membership(circle_id, new_owner_id).await?;

        // Update circle owner
        sqlx::query("UPDATE circles SET owner_id = $2 WHERE id = $1")
            .bind(circle_id)
            .bind(new_owner_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        // Swap roles in circle_members
        sqlx::query(
            "UPDATE circle_members SET role = 'member' WHERE circle_id = $1 AND user_id = $2",
        )
        .bind(circle_id)
        .bind(requester_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        sqlx::query(
            "UPDATE circle_members SET role = 'owner' WHERE circle_id = $1 AND user_id = $2",
        )
        .bind(circle_id)
        .bind(new_owner_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        self.bump_circle_version(circle_id);

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn list_reservations(&self, user_id: Uuid) -> Result<Vec<ReservationResponse>, AppError> {
        let rows = sqlx::query_as::<
            _,
            (
                Uuid,
                String,
                Option<String>,
                Option<rust_decimal::Decimal>,
                String,
                String,
                Option<String>,
                Uuid,
                Option<String>,
                chrono::DateTime<chrono::Utc>,
            ),
        >(
            "SELECT DISTINCT ON (i.id) \
               i.id, i.name, i.image_url, i.estimated_price, i.status, \
               COALESCE(u.display_name, u.username, '') as owner_name, u.avatar_url, \
               c.id as circle_id, c.name as circle_name, \
               i.claimed_at \
             FROM items i \
             JOIN circle_items ci ON ci.item_id = i.id \
             JOIN circles c ON c.id = ci.circle_id \
             JOIN circle_members cm ON cm.circle_id = c.id AND cm.user_id = $1 \
             JOIN users u ON u.id = i.user_id \
             WHERE i.claimed_by = $1 \
               AND i.status IN ('active', 'purchased') \
             ORDER BY i.id, i.claimed_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        let reservations = rows
            .into_iter()
            .map(
                |(
                    item_id,
                    item_name,
                    item_image_url,
                    price,
                    status,
                    owner_name,
                    owner_avatar,
                    cid,
                    cname,
                    claimed_at,
                )| {
                    ReservationResponse {
                        item_id,
                        item_name,
                        item_image_url,
                        item_estimated_price: price,
                        item_status: status,
                        owner_name,
                        owner_avatar_url: owner_avatar,
                        circle_id: cid,
                        circle_name: cname,
                        claimed_at,
                    }
                },
            )
            .collect();

        Ok(reservations)
    }
}
