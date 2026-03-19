use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use crate::dto::community_wishes::{
    AdminWishResponse, MyWishResponse, WishDetailResponse, WishResponse,
};
use crate::dto::pagination::PaginatedResponse;
use crate::errors::AppError;
use crate::models::community_wish::WishStatus;
use crate::services::moderation_service::ModerationResult;
use crate::traits::{self, NotificationRequest};

// ── Configuration constants ──────────────────────────────────────────

const MAX_ACTIVE_WISHES_PER_USER: i64 = 3;
const MIN_ACCOUNT_AGE_HOURS: i64 = 24;
const WISH_REPORT_THRESHOLD: i32 = 5;
const MAX_REPORTS_PER_USER_PER_DAY: i64 = 10;
const MAX_REOPEN_COUNT: i32 = 2;
const REOPEN_COOLDOWN_HOURS: i64 = 24;
const CACHE_TTL_SECS: u64 = 60;

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgCommunityWishService {
    #[allow(dead_code)]
    pool: sqlx::PgPool,
    wish_repo: Arc<dyn traits::CommunityWishRepo>,
    report_repo: Arc<dyn traits::WishReportRepo>,
    user_repo: Arc<dyn traits::UserRepo>,
    push_token_repo: Arc<dyn traits::PushTokenRepo>,
    notification_svc: Arc<dyn traits::NotificationService>,
    notification_repo: Arc<dyn traits::NotificationRepo>,
    moderation_svc: Arc<dyn traits::ModerationService>,
    redis: redis::Client,
}

impl PgCommunityWishService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pool: sqlx::PgPool,
        wish_repo: Arc<dyn traits::CommunityWishRepo>,
        report_repo: Arc<dyn traits::WishReportRepo>,
        user_repo: Arc<dyn traits::UserRepo>,
        push_token_repo: Arc<dyn traits::PushTokenRepo>,
        notification_svc: Arc<dyn traits::NotificationService>,
        notification_repo: Arc<dyn traits::NotificationRepo>,
        moderation_svc: Arc<dyn traits::ModerationService>,
        redis: redis::Client,
    ) -> Self {
        Self {
            pool,
            wish_repo,
            report_repo,
            user_repo,
            push_token_repo,
            notification_svc,
            notification_repo,
            moderation_svc,
            redis,
        }
    }

    fn notify_user(
        &self,
        user_id: Uuid,
        title: String,
        body: String,
        notif_type: &str,
        wish_id: Option<Uuid>,
        actor_id: Option<Uuid>,
    ) {
        let push_token_repo = self.push_token_repo.clone();
        let notification_svc = self.notification_svc.clone();
        let notification_repo = self.notification_repo.clone();
        let notif_type = notif_type.to_string();

        tokio::spawn(async move {
            // Persist to notification center
            let _ = notification_repo
                .create(
                    user_id,
                    &notif_type,
                    &title,
                    &body,
                    None,    // circle_id
                    None,    // item_id
                    wish_id, // wish_id
                    actor_id,
                )
                .await;

            // Send push notification
            let tokens = match push_token_repo.find_by_user(user_id).await {
                Ok(t) => t,
                Err(e) => {
                    tracing::warn!(%user_id, error = %e, "failed to fetch push tokens for wish notification");
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

    async fn bump_cache_version(&self) {
        if let Ok(mut conn) = self.redis.get_multiplexed_async_connection().await {
            let _: Result<i64, _> = redis::cmd("INCR")
                .arg("community_wishes:ver")
                .query_async(&mut conn)
                .await;
        }
    }

    /// Spawn an async moderation check for a wish. Sets status to `pending` first,
    /// then runs moderation with retries. On completion: `open` or `flagged`.
    /// Used by create_wish, update_wish, and reopen_wish.
    #[allow(clippy::too_many_arguments)]
    fn spawn_moderation_check(
        &self,
        wish_id: Uuid,
        owner_id: Uuid,
        title: &str,
        description: Option<&str>,
        category: &str,
        image_url: Option<&str>,
        links: Option<&[String]>,
    ) {
        let mod_title = title.to_string();
        let mod_desc = description.map(|d| d.to_string());
        let mod_cat = category.to_string();
        let mod_image_url = image_url.map(|u| u.to_string());
        let mod_links: Option<Vec<String>> = links.map(|l| l.to_vec());
        let moderation_svc = self.moderation_svc.clone();
        let wish_repo = self.wish_repo.clone();
        let push_token_repo = self.push_token_repo.clone();
        let notification_svc = self.notification_svc.clone();
        let notification_repo = self.notification_repo.clone();
        let redis = self.redis.clone();

        tokio::spawn(async move {
            let max_retries = 3;
            let mut result = None;

            for attempt in 0..max_retries {
                match moderation_svc
                    .check_wish(
                        &mod_title,
                        mod_desc.as_deref(),
                        &mod_cat,
                        mod_image_url.as_deref(),
                        mod_links.as_deref(),
                    )
                    .await
                {
                    Ok(r) => {
                        result = Some(r);
                        break;
                    }
                    Err(e) => {
                        tracing::warn!(
                            wish_id = %wish_id,
                            attempt = attempt + 1,
                            error = %e,
                            "moderation check failed, retrying"
                        );
                        if attempt < max_retries - 1 {
                            tokio::time::sleep(std::time::Duration::from_secs(
                                2u64.pow(attempt as u32),
                            ))
                            .await;
                        }
                    }
                }
            }

            let (new_status, note, notif_title, notif_body) = match result {
                Some(ModerationResult::Approved) => (
                    WishStatus::Open,
                    None,
                    "Souhait publié !".to_string(),
                    "Votre souhait est maintenant visible sur le mur d'entraide.".to_string(),
                ),
                Some(ModerationResult::Flagged { reason }) => (
                    WishStatus::Flagged,
                    Some(reason),
                    "Souhait en révision".to_string(),
                    "Votre souhait est en cours de révision par notre équipe.".to_string(),
                ),
                None => (
                    WishStatus::Flagged,
                    Some("moderation service unavailable after retries".to_string()),
                    "Souhait en révision".to_string(),
                    "Votre souhait est en cours de révision par notre équipe.".to_string(),
                ),
            };

            if let Err(e) = wish_repo
                .update_status(wish_id, new_status, note.as_deref())
                .await
            {
                tracing::error!(wish_id = %wish_id, error = %e, "failed to update wish status after moderation");
                return;
            }

            // Bump cache
            if let Ok(mut conn) = redis.get_multiplexed_async_connection().await {
                let _: Result<i64, _> = redis::cmd("INCR")
                    .arg("community_wishes:ver")
                    .query_async(&mut conn)
                    .await;
            }

            // Persist notification
            let notif_type = if new_status == WishStatus::Open {
                "wish_moderation_approved"
            } else {
                "wish_moderation_flagged"
            };
            let _ = notification_repo
                .create(
                    owner_id,
                    notif_type,
                    &notif_title,
                    &notif_body,
                    None,
                    None,
                    Some(wish_id),
                    None,
                )
                .await;

            // Push notification
            let tokens = match push_token_repo.find_by_user(owner_id).await {
                Ok(t) => t,
                Err(_) => return,
            };
            let requests: Vec<NotificationRequest> = tokens
                .into_iter()
                .map(|pt| NotificationRequest {
                    device_token: pt.token,
                    title: notif_title.clone(),
                    body: notif_body.clone(),
                    custom_data: std::collections::HashMap::new(),
                    ..Default::default()
                })
                .collect();
            if !requests.is_empty() {
                notification_svc.send_batch(&requests).await;
            }
        });
    }

    async fn get_cache_version(&self) -> Option<i64> {
        let Ok(mut conn) = self.redis.get_multiplexed_async_connection().await else {
            return None;
        };
        let ver: Result<i64, _> = redis::cmd("GET")
            .arg("community_wishes:ver")
            .query_async(&mut conn)
            .await;
        Some(ver.unwrap_or(0))
    }

    async fn get_cached(&self, cache_key: &str) -> Option<String> {
        let Ok(mut conn) = self.redis.get_multiplexed_async_connection().await else {
            return None;
        };
        redis::cmd("GET")
            .arg(cache_key)
            .query_async(&mut conn)
            .await
            .ok()
    }

    async fn set_cached(&self, cache_key: &str, data: &str) {
        if let Ok(mut conn) = self.redis.get_multiplexed_async_connection().await {
            let _: Result<(), _> = redis::cmd("SET")
                .arg(cache_key)
                .arg(data)
                .arg("EX")
                .arg(CACHE_TTL_SECS)
                .query_async(&mut conn)
                .await;
        }
    }

    async fn check_account_age(&self, user_id: Uuid) -> Result<(), AppError> {
        let created_at = self
            .user_repo
            .get_user_created_at(user_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("user not found".into()))?;

        let age = Utc::now() - created_at;
        if age.num_hours() < MIN_ACCOUNT_AGE_HOURS {
            return Err(AppError::Forbidden(
                "account must be at least 24 hours old".into(),
            ));
        }
        Ok(())
    }

    async fn get_display_name(&self, user_id: Uuid) -> Option<String> {
        self.user_repo
            .find_by_id(user_id)
            .await
            .ok()
            .flatten()
            .and_then(|u| u.display_name)
    }

    fn to_wish_response(
        &self,
        wish: &crate::models::community_wish::CommunityWish,
        caller_id: Option<Uuid>,
        owner_display_name: Option<String>,
    ) -> WishResponse {
        let display_name = if wish.is_anonymous {
            None
        } else {
            owner_display_name
        };

        WishResponse {
            id: wish.id,
            display_name,
            title: wish.title.clone(),
            description: wish.description.clone(),
            category: wish.category.clone(),
            status: wish.status.clone(),
            is_mine: caller_id == Some(wish.owner_id),
            is_matched_by_me: caller_id.is_some() && wish.matched_with == caller_id,
            image_url: wish.image_url.clone(),
            links: wish.links.clone(),
            created_at: wish.created_at,
        }
    }
}

#[async_trait]
impl traits::CommunityWishService for PgCommunityWishService {
    #[tracing::instrument(skip(self, description))]
    async fn create_wish(
        &self,
        user_id: Uuid,
        title: &str,
        description: Option<&str>,
        category: &str,
        is_anonymous: bool,
        image_url: Option<&str>,
        links: Option<&[String]>,
    ) -> Result<MyWishResponse, AppError> {
        // Guard: account age
        self.check_account_age(user_id).await?;

        // Guard: max active wishes
        let active_count = self
            .wish_repo
            .count_active_by_owner(user_id)
            .await
            .map_err(AppError::Internal)?;
        if active_count >= MAX_ACTIVE_WISHES_PER_USER {
            return Err(AppError::Conflict(
                "you already have 3 active wishes".into(),
            ));
        }

        // Guard: non-anonymous requires display_name
        if !is_anonymous {
            let user = self
                .user_repo
                .find_by_id(user_id)
                .await
                .map_err(AppError::Internal)?
                .ok_or_else(|| AppError::NotFound("user not found".into()))?;
            if user.display_name.is_none() {
                return Err(AppError::BadRequest(
                    "please set a display name in your profile before creating a non-anonymous wish"
                        .into(),
                ));
            }
        }

        // Create with pending status
        let wish = self
            .wish_repo
            .create(
                user_id,
                title,
                description,
                category,
                is_anonymous,
                image_url,
                links,
            )
            .await
            .map_err(AppError::Internal)?;

        // Async moderation check
        self.spawn_moderation_check(
            wish.id,
            user_id,
            title,
            description,
            category,
            image_url,
            links,
        );

        Ok(MyWishResponse {
            id: wish.id,
            title: wish.title,
            description: wish.description,
            category: wish.category,
            status: wish.status,
            is_anonymous: wish.is_anonymous,
            matched_with_display_name: None,
            report_count: wish.report_count,
            reopen_count: wish.reopen_count,
            moderation_note: wish.moderation_note,
            image_url: wish.image_url,
            links: wish.links,
            created_at: wish.created_at,
            matched_at: wish.matched_at,
            fulfilled_at: wish.fulfilled_at,
            closed_at: wish.closed_at,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn list_wishes(
        &self,
        caller_id: Option<Uuid>,
        category: Option<&str>,
        page: i64,
        limit: i64,
        offset: i64,
    ) -> Result<PaginatedResponse<WishResponse>, AppError> {
        // Check cache
        let query_hash = format!("{:?}:{limit}:{offset}", category);
        if let Some(ver) = self.get_cache_version().await {
            let cache_key = format!("community_wishes:v{ver}:{query_hash}");
            if caller_id.is_none()
                && let Some(cached) = self.get_cached(&cache_key).await
                && let Ok(resp) = serde_json::from_str::<PaginatedResponse<WishResponse>>(&cached)
            {
                return Ok(resp);
            }
        }

        let wishes = self
            .wish_repo
            .list_open(category, limit, offset)
            .await
            .map_err(AppError::Internal)?;
        let total = self
            .wish_repo
            .count_open(category)
            .await
            .map_err(AppError::Internal)?;

        // Batch-fetch owner display names (dedup to reduce query size)
        let owner_ids: Vec<Uuid> = wishes
            .iter()
            .map(|w| w.owner_id)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        let users = self
            .user_repo
            .find_by_ids(&owner_ids)
            .await
            .map_err(AppError::Internal)?;
        let user_map: std::collections::HashMap<Uuid, Option<String>> =
            users.into_iter().map(|u| (u.id, u.display_name)).collect();

        let wish_responses: Vec<WishResponse> = wishes
            .iter()
            .map(|w| {
                let dn = user_map.get(&w.owner_id).cloned().flatten();
                self.to_wish_response(w, caller_id, dn)
            })
            .collect();

        let resp = PaginatedResponse::new(wish_responses, total, page, limit);

        // Cache only for unauthenticated (public) requests
        if caller_id.is_none()
            && let Some(ver) = self.get_cache_version().await
        {
            let cache_key = format!("community_wishes:v{ver}:{query_hash}");
            if let Ok(json) = serde_json::to_string(&resp) {
                self.set_cached(&cache_key, &json).await;
            }
        }

        Ok(resp)
    }

    #[tracing::instrument(skip(self))]
    async fn get_wish(
        &self,
        wish_id: Uuid,
        caller_id: Option<Uuid>,
    ) -> Result<WishDetailResponse, AppError> {
        let wish = self
            .wish_repo
            .find_by_id(wish_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("wish not found".into()))?;

        // Only show publicly visible statuses (or if caller is owner/donor)
        let status = WishStatus::parse(&wish.status);
        let is_owner = caller_id == Some(wish.owner_id);
        let is_donor = caller_id.is_some() && wish.matched_with == caller_id;

        match status {
            Some(WishStatus::Open | WishStatus::Matched) => {}
            _ if is_owner || is_donor => {}
            _ => return Err(AppError::NotFound("wish not found".into())),
        }

        let owner_dn = self.get_display_name(wish.owner_id).await;
        let display_name = if wish.is_anonymous && !is_owner && !is_donor {
            None
        } else {
            owner_dn
        };

        // Show donor display_name only to owner and donor
        let matched_with_dn = if (is_owner || is_donor) && wish.matched_with.is_some() {
            if let Some(donor_id) = wish.matched_with {
                self.get_display_name(donor_id).await
            } else {
                None
            }
        } else {
            None
        };

        Ok(WishDetailResponse {
            id: wish.id,
            display_name,
            title: wish.title,
            description: wish.description,
            category: wish.category,
            status: wish.status,
            is_mine: is_owner,
            is_matched_by_me: is_donor,
            matched_with_display_name: matched_with_dn,
            image_url: wish.image_url,
            links: wish.links,
            matched_at: wish.matched_at,
            fulfilled_at: wish.fulfilled_at,
            created_at: wish.created_at,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn list_my_wishes(&self, user_id: Uuid) -> Result<Vec<MyWishResponse>, AppError> {
        let wishes = self
            .wish_repo
            .list_by_owner(user_id)
            .await
            .map_err(AppError::Internal)?;

        let mut responses = Vec::with_capacity(wishes.len());
        for wish in &wishes {
            let donor_dn = if let Some(donor_id) = wish.matched_with {
                self.get_display_name(donor_id).await
            } else {
                None
            };
            responses.push(MyWishResponse {
                id: wish.id,
                title: wish.title.clone(),
                description: wish.description.clone(),
                category: wish.category.clone(),
                status: wish.status.clone(),
                is_anonymous: wish.is_anonymous,
                matched_with_display_name: donor_dn,
                report_count: wish.report_count,
                reopen_count: wish.reopen_count,
                moderation_note: wish.moderation_note.clone(),
                image_url: wish.image_url.clone(),
                links: wish.links.clone(),
                created_at: wish.created_at,
                matched_at: wish.matched_at,
                fulfilled_at: wish.fulfilled_at,
                closed_at: wish.closed_at,
            });
        }
        Ok(responses)
    }

    #[tracing::instrument(skip(self))]
    async fn update_wish(
        &self,
        wish_id: Uuid,
        user_id: Uuid,
        title: Option<&str>,
        description: Option<&str>,
        category: Option<&str>,
        image_url: Option<&str>,
        links: Option<&[String]>,
    ) -> Result<MyWishResponse, AppError> {
        let wish = self
            .wish_repo
            .find_by_id(wish_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("wish not found".into()))?;

        if wish.owner_id != user_id {
            return Err(AppError::Forbidden("not the wish owner".into()));
        }

        let status = WishStatus::parse(&wish.status)
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("invalid wish status")))?;
        if !matches!(status, WishStatus::Open | WishStatus::Review) {
            return Err(AppError::BadRequest(
                "can only update wishes in open or review status".into(),
            ));
        }

        let updated = self
            .wish_repo
            .update_content(wish_id, title, description, category, image_url, links)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("wish not found".into()))?;

        // Set to pending and re-run moderation on updated content
        self.wish_repo
            .update_status(wish_id, WishStatus::Pending, None)
            .await
            .map_err(AppError::Internal)?;

        self.spawn_moderation_check(
            wish_id,
            user_id,
            updated.title.as_str(),
            updated.description.as_deref(),
            &updated.category,
            updated.image_url.as_deref(),
            updated.links.as_deref(),
        );

        self.bump_cache_version().await;

        let donor_dn = if let Some(donor_id) = updated.matched_with {
            self.get_display_name(donor_id).await
        } else {
            None
        };

        Ok(MyWishResponse {
            id: updated.id,
            title: updated.title,
            description: updated.description,
            category: updated.category,
            status: "pending".to_string(), // reflect the actual new status
            is_anonymous: updated.is_anonymous,
            matched_with_display_name: donor_dn,
            report_count: updated.report_count,
            reopen_count: updated.reopen_count,
            moderation_note: updated.moderation_note,
            image_url: updated.image_url,
            links: updated.links,
            created_at: updated.created_at,
            matched_at: updated.matched_at,
            fulfilled_at: updated.fulfilled_at,
            closed_at: updated.closed_at,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn close_wish(&self, wish_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let wish = self
            .wish_repo
            .find_by_id(wish_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("wish not found".into()))?;

        if wish.owner_id != user_id {
            return Err(AppError::Forbidden("not the wish owner".into()));
        }

        let status = WishStatus::parse(&wish.status)
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("invalid wish status")))?;
        if status.is_terminal() {
            return Err(AppError::BadRequest(
                "wish is already in a terminal state".into(),
            ));
        }

        // If matched, notify donor
        if status == WishStatus::Matched
            && let Some(donor_id) = wish.matched_with
        {
            self.notify_user(
                donor_id,
                "Souhait fermé".into(),
                "L'auteur a fermé son souhait.".into(),
                "wish_closed",
                Some(wish_id),
                Some(user_id),
            );
        }

        self.wish_repo
            .set_closed(wish_id, Utc::now())
            .await
            .map_err(AppError::Internal)?;

        self.bump_cache_version().await;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn reopen_wish(&self, wish_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let wish = self
            .wish_repo
            .find_by_id(wish_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("wish not found".into()))?;

        if wish.owner_id != user_id {
            return Err(AppError::Forbidden("not the wish owner".into()));
        }

        let status = WishStatus::parse(&wish.status)
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("invalid wish status")))?;
        if status != WishStatus::Review {
            return Err(AppError::BadRequest(
                "can only reopen wishes in review status".into(),
            ));
        }

        if wish.reopen_count >= MAX_REOPEN_COUNT {
            return Err(AppError::Forbidden(
                "maximum number of reopens reached, please wait for admin review".into(),
            ));
        }

        // Check cooldown
        if let Some(last_reopen) = wish.last_reopen_at {
            let elapsed = Utc::now() - last_reopen;
            if elapsed.num_hours() < REOPEN_COOLDOWN_HOURS {
                return Err(AppError::BadRequest(
                    "please wait 24 hours before reopening again".into(),
                ));
            }
        }

        // Increment reopen count and clear reports
        self.wish_repo
            .increment_reopen_count(wish_id, Utc::now())
            .await
            .map_err(AppError::Internal)?;

        self.wish_repo
            .reset_reports(wish_id)
            .await
            .map_err(AppError::Internal)?;

        self.report_repo
            .delete_by_wish(wish_id)
            .await
            .map_err(AppError::Internal)?;

        // Set to pending and re-run moderation on current content
        self.wish_repo
            .update_status(wish_id, WishStatus::Pending, None)
            .await
            .map_err(AppError::Internal)?;

        self.spawn_moderation_check(
            wish_id,
            user_id,
            &wish.title,
            wish.description.as_deref(),
            &wish.category,
            wish.image_url.as_deref(),
            wish.links.as_deref(),
        );

        self.bump_cache_version().await;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn offer_wish(&self, wish_id: Uuid, donor_id: Uuid) -> Result<(), AppError> {
        self.check_account_age(donor_id).await?;

        let wish = self
            .wish_repo
            .find_by_id(wish_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("wish not found".into()))?;

        if wish.owner_id == donor_id {
            return Err(AppError::BadRequest("cannot offer on your own wish".into()));
        }

        let status = WishStatus::parse(&wish.status)
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("invalid wish status")))?;
        if status != WishStatus::Open {
            return Err(AppError::BadRequest("wish is not open for offers".into()));
        }

        let matched = self
            .wish_repo
            .set_matched(wish_id, donor_id, Utc::now())
            .await
            .map_err(AppError::Internal)?;
        if !matched {
            return Err(AppError::Conflict(
                "wish was already matched by another donor".into(),
            ));
        }

        self.bump_cache_version().await;

        // Notify owner
        let donor_name = self
            .get_display_name(donor_id)
            .await
            .unwrap_or_else(|| "Quelqu'un".to_string());
        self.notify_user(
            wish.owner_id,
            "Offre d'aide !".into(),
            format!("{donor_name} propose de vous aider !"),
            "wish_offer",
            Some(wish_id),
            Some(donor_id),
        );

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn withdraw_offer(&self, wish_id: Uuid, donor_id: Uuid) -> Result<(), AppError> {
        let wish = self
            .wish_repo
            .find_by_id(wish_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("wish not found".into()))?;

        let status = WishStatus::parse(&wish.status)
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("invalid wish status")))?;
        if status != WishStatus::Matched {
            return Err(AppError::BadRequest("wish is not in matched status".into()));
        }

        if wish.matched_with != Some(donor_id) {
            return Err(AppError::Forbidden("you are not the matched donor".into()));
        }

        self.wish_repo
            .clear_match(wish_id)
            .await
            .map_err(AppError::Internal)?;

        self.bump_cache_version().await;

        // Notify owner
        self.notify_user(
            wish.owner_id,
            "Offre retirée".into(),
            "L'offre d'aide a été retirée.".into(),
            "wish_offer_withdrawn",
            Some(wish_id),
            Some(donor_id),
        );

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn reject_offer(&self, wish_id: Uuid, owner_id: Uuid) -> Result<(), AppError> {
        let wish = self
            .wish_repo
            .find_by_id(wish_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("wish not found".into()))?;

        if wish.owner_id != owner_id {
            return Err(AppError::Forbidden("not the wish owner".into()));
        }

        let status = WishStatus::parse(&wish.status)
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("invalid wish status")))?;
        if status != WishStatus::Matched {
            return Err(AppError::BadRequest("wish is not in matched status".into()));
        }

        let donor_id = wish.matched_with;

        self.wish_repo
            .clear_match(wish_id)
            .await
            .map_err(AppError::Internal)?;

        self.bump_cache_version().await;

        // Notify donor
        if let Some(did) = donor_id {
            self.notify_user(
                did,
                "Offre déclinée".into(),
                "L'auteur a décliné votre offre d'aide.".into(),
                "wish_offer_rejected",
                Some(wish_id),
                Some(owner_id),
            );
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn confirm_wish(&self, wish_id: Uuid, owner_id: Uuid) -> Result<(), AppError> {
        let wish = self
            .wish_repo
            .find_by_id(wish_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("wish not found".into()))?;

        if wish.owner_id != owner_id {
            return Err(AppError::Forbidden("not the wish owner".into()));
        }

        let status = WishStatus::parse(&wish.status)
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("invalid wish status")))?;
        if status != WishStatus::Matched {
            return Err(AppError::BadRequest("wish is not in matched status".into()));
        }

        self.wish_repo
            .set_fulfilled(wish_id, Utc::now())
            .await
            .map_err(AppError::Internal)?;

        self.bump_cache_version().await;

        // Notify donor
        if let Some(donor_id) = wish.matched_with {
            self.notify_user(
                donor_id,
                "Don confirmé !".into(),
                "Votre don a été confirmé, merci !".into(),
                "wish_confirmed",
                Some(wish_id),
                Some(owner_id),
            );
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn report_wish(
        &self,
        wish_id: Uuid,
        reporter_id: Uuid,
        reason: &str,
    ) -> Result<(), AppError> {
        self.check_account_age(reporter_id).await?;

        let wish = self
            .wish_repo
            .find_by_id(wish_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("wish not found".into()))?;

        if wish.owner_id == reporter_id {
            return Err(AppError::BadRequest("cannot report your own wish".into()));
        }

        let status = WishStatus::parse(&wish.status)
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("invalid wish status")))?;
        if status != WishStatus::Open {
            return Err(AppError::BadRequest("can only report open wishes".into()));
        }

        // Check daily report limit
        let today_count = self
            .report_repo
            .count_by_reporter_today(reporter_id)
            .await
            .map_err(AppError::Internal)?;
        if today_count >= MAX_REPORTS_PER_USER_PER_DAY {
            return Err(AppError::BadRequest("daily report limit reached".into()));
        }

        // Check if already reported
        let already = self
            .report_repo
            .has_reported(wish_id, reporter_id)
            .await
            .map_err(AppError::Internal)?;
        if already {
            return Err(AppError::Conflict(
                "you have already reported this wish".into(),
            ));
        }

        // Create report
        self.report_repo
            .create(wish_id, reporter_id, reason)
            .await
            .map_err(AppError::Internal)?;

        // Increment report count and check threshold
        let new_count = self
            .wish_repo
            .increment_report_count(wish_id)
            .await
            .map_err(AppError::Internal)?;

        if new_count >= WISH_REPORT_THRESHOLD {
            // Auto-moderate: move to review
            self.wish_repo
                .update_status(
                    wish_id,
                    WishStatus::Review,
                    Some("auto-moderated: report threshold reached"),
                )
                .await
                .map_err(AppError::Internal)?;

            self.bump_cache_version().await;

            // Notify owner
            self.notify_user(
                wish.owner_id,
                "Souhait signalé".into(),
                "Votre souhait a été signalé par la communauté et est en cours de révision.".into(),
                "wish_reported",
                Some(wish_id),
                None,
            );
        }

        Ok(())
    }

    // ── Admin actions ────────────────────────────────────────────────

    #[tracing::instrument(skip(self))]
    async fn admin_list_flagged(
        &self,
        page: i64,
        limit: i64,
        offset: i64,
    ) -> Result<PaginatedResponse<AdminWishResponse>, AppError> {
        let wishes = self
            .wish_repo
            .list_flagged(limit, offset)
            .await
            .map_err(AppError::Internal)?;
        let total = self
            .wish_repo
            .count_flagged()
            .await
            .map_err(AppError::Internal)?;

        let items: Vec<AdminWishResponse> = wishes
            .into_iter()
            .map(|w| AdminWishResponse {
                id: w.id,
                owner_id: w.owner_id,
                title: w.title,
                description: w.description,
                category: w.category,
                status: w.status,
                moderation_note: w.moderation_note,
                image_url: w.image_url,
                links: w.links,
                report_count: w.report_count,
                created_at: w.created_at,
            })
            .collect();

        Ok(PaginatedResponse::new(items, total, page, limit))
    }

    #[tracing::instrument(skip(self))]
    async fn admin_approve(&self, wish_id: Uuid) -> Result<(), AppError> {
        let wish = self
            .wish_repo
            .find_by_id(wish_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("wish not found".into()))?;

        let status = WishStatus::parse(&wish.status)
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("invalid wish status")))?;
        if !matches!(status, WishStatus::Flagged | WishStatus::Review) {
            return Err(AppError::BadRequest(
                "can only approve flagged or review wishes".into(),
            ));
        }

        self.wish_repo
            .update_status(wish_id, WishStatus::Open, None)
            .await
            .map_err(AppError::Internal)?;

        self.bump_cache_version().await;

        self.notify_user(
            wish.owner_id,
            "Souhait approuvé".into(),
            "Votre souhait a été approuvé et est maintenant visible.".into(),
            "wish_approved",
            Some(wish_id),
            None,
        );

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn admin_reject(&self, wish_id: Uuid) -> Result<(), AppError> {
        let wish = self
            .wish_repo
            .find_by_id(wish_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("wish not found".into()))?;

        let status = WishStatus::parse(&wish.status)
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("invalid wish status")))?;
        if !matches!(status, WishStatus::Flagged | WishStatus::Review) {
            return Err(AppError::BadRequest(
                "can only reject flagged or review wishes".into(),
            ));
        }

        self.wish_repo
            .update_status(wish_id, WishStatus::Rejected, None)
            .await
            .map_err(AppError::Internal)?;

        self.bump_cache_version().await;

        self.notify_user(
            wish.owner_id,
            "Souhait refusé".into(),
            "Votre souhait n'a pas été retenu par notre équipe.".into(),
            "wish_rejected",
            Some(wish_id),
            None,
        );

        Ok(())
    }
}
