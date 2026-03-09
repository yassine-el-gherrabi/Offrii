use std::sync::Arc;

use async_trait::async_trait;
use chrono::{NaiveTime, Timelike};
use chrono_tz::Tz;
use uuid::Uuid;

use crate::dto::categories::CategoryResponse;
use crate::dto::items::ItemResponse;
use crate::dto::users::{UpdateProfileRequest, UserDataExport, UserProfileResponse};
use crate::errors::AppError;
use crate::traits;

/// Validate username format: ^[a-z][a-z0-9_]{2,29}$
fn is_valid_username(s: &str) -> bool {
    if s.len() < 3 || s.len() > 30 {
        return false;
    }
    let mut chars = s.chars();
    let first = match chars.next() {
        Some(c) if c.is_ascii_lowercase() => c,
        _ => return false,
    };
    let _ = first;
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgUserService {
    user_repo: Arc<dyn traits::UserRepo>,
    item_repo: Arc<dyn traits::ItemRepo>,
    category_repo: Arc<dyn traits::CategoryRepo>,
}

impl PgUserService {
    pub fn new(
        user_repo: Arc<dyn traits::UserRepo>,
        item_repo: Arc<dyn traits::ItemRepo>,
        category_repo: Arc<dyn traits::CategoryRepo>,
    ) -> Self {
        Self {
            user_repo,
            item_repo,
            category_repo,
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
        if req.display_name.is_none()
            && req.username.is_none()
            && req.reminder_freq.is_none()
            && req.reminder_time.is_none()
            && req.timezone.is_none()
            && req.locale.is_none()
        {
            return self.get_profile(user_id).await;
        }

        // Validate username if provided
        if let Some(ref username) = req.username {
            if !is_valid_username(username) {
                return Err(AppError::BadRequest(
                    "username must be 3-30 characters, start with a letter, and contain only lowercase letters, digits, and underscores".into(),
                ));
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

        // Validate reminder_freq if provided
        if let Some(ref freq) = req.reminder_freq {
            const VALID_FREQS: &[&str] = &["never", "daily", "weekly", "monthly"];
            if !VALID_FREQS.contains(&freq.as_str()) {
                return Err(AppError::BadRequest(
                    "reminder_freq must be one of: never, daily, weekly, monthly".into(),
                ));
            }
        }

        // Validate timezone if provided
        if let Some(ref tz) = req.timezone {
            tz.parse::<Tz>()
                .map_err(|_| AppError::BadRequest(format!("invalid timezone: {tz}")))?;
        }

        // Compute utc_reminder_hour if timezone or reminder_time changed
        let utc_hour = if req.timezone.is_some() || req.reminder_time.is_some() {
            // Need current user data to fill in unchanged fields
            let current = self
                .user_repo
                .find_by_id(user_id)
                .await
                .map_err(AppError::Internal)?
                .ok_or_else(|| AppError::NotFound("user not found".into()))?;

            let tz_str = req.timezone.as_deref().unwrap_or(&current.timezone);
            let time = req.reminder_time.unwrap_or(current.reminder_time);

            Some(compute_utc_hour(time, tz_str)?)
        } else {
            None
        };

        let user = self
            .user_repo
            .update_profile(
                user_id,
                req.display_name.as_deref(),
                req.username.as_deref(),
                req.reminder_freq.as_deref(),
                req.reminder_time,
                req.timezone.as_deref(),
                utc_hour,
                req.locale.as_deref(),
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
            .list_by_user(user_id)
            .await
            .map_err(AppError::Internal)?;

        Ok(UserDataExport {
            profile: UserProfileResponse::from(&user),
            items: items.into_iter().map(ItemResponse::from).collect(),
            categories: categories.into_iter().map(CategoryResponse::from).collect(),
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

/// Convert a local reminder time + timezone into the UTC hour (0-23).
fn compute_utc_hour(local_time: NaiveTime, timezone: &str) -> Result<i16, AppError> {
    use chrono::TimeZone;

    let tz: Tz = timezone
        .parse()
        .map_err(|_| AppError::BadRequest(format!("invalid timezone: {timezone}")))?;

    // Use today's date to get current offset (handles DST)
    let today = chrono::Utc::now().date_naive();
    let local_dt = tz
        .from_local_datetime(&today.and_time(local_time))
        .earliest()
        .ok_or_else(|| {
            AppError::BadRequest("ambiguous or invalid local time for timezone".into())
        })?;

    let utc_dt = local_dt.with_timezone(&chrono::Utc);
    Ok(utc_dt.time().hour() as i16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_utc_hour_utc() {
        let time = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        let hour = compute_utc_hour(time, "UTC").unwrap();
        assert_eq!(hour, 10);
    }

    #[test]
    fn compute_utc_hour_paris() {
        let time = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        let hour = compute_utc_hour(time, "Europe/Paris").unwrap();
        // Paris is UTC+1 (winter) or UTC+2 (summer)
        assert!(hour == 8 || hour == 9, "expected 8 or 9, got {hour}");
    }

    #[test]
    fn invalid_timezone_error() {
        let time = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        let result = compute_utc_hour(time, "Invalid/Timezone");
        assert!(result.is_err());
    }

    #[test]
    fn compute_utc_hour_new_york() {
        // America/New_York is UTC-5 (winter) or UTC-4 (summer)
        let time = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        let hour = compute_utc_hour(time, "America/New_York").unwrap();
        assert!(hour == 14 || hour == 15, "expected 14 or 15, got {hour}");
    }

    #[test]
    fn compute_utc_hour_midnight_crossing() {
        // Pacific/Auckland is UTC+12/+13 — 2:00 local → should wrap to previous day UTC
        let time = NaiveTime::from_hms_opt(2, 0, 0).unwrap();
        let hour = compute_utc_hour(time, "Pacific/Auckland").unwrap();
        // 2:00 NZST (UTC+12) → 14:00 UTC previous day
        // 2:00 NZDT (UTC+13) → 13:00 UTC previous day
        assert!(hour == 13 || hour == 14, "expected 13 or 14, got {hour}");
    }
}
