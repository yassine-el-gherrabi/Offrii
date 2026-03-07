use std::sync::Arc;

use async_trait::async_trait;
use chrono::{TimeDelta, Utc};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::traits;

// ── Scoring constants ────────────────────────────────────────────────

const PRIORITY_WEIGHT: f64 = 10.0;
const AGE_WEIGHT: f64 = 1.0;
const MIN_AGE_DAYS: i64 = 7;
const TOP_N: usize = 3;

fn score(priority: i16, days_waiting: i64) -> f64 {
    PRIORITY_WEIGHT * (4 - priority) as f64 + AGE_WEIGHT * days_waiting as f64
}

/// TTL in seconds for the anti-spam Redis key, based on reminder frequency.
fn freq_ttl(freq: &str) -> u64 {
    match freq {
        "daily" => 82_800,      // 23h
        "weekly" => 561_600,    // 6d 12h
        "monthly" => 2_505_600, // 29d
        _ => 82_800,            // fallback to daily
    }
}

// ── Expo Push API types ──────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct ExpoPushMessage {
    to: String,
    title: String,
    body: String,
    sound: &'static str,
}

#[derive(Debug, Deserialize)]
struct ExpoPushResponse {
    data: Vec<ExpoPushTicket>,
}

#[derive(Debug, Deserialize)]
struct ExpoPushTicket {
    status: String,
    #[serde(default)]
    details: Option<ExpoPushDetails>,
}

#[derive(Debug, Deserialize)]
struct ExpoPushDetails {
    #[serde(default)]
    error: Option<String>,
}

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgReminderService {
    user_repo: Arc<dyn traits::UserRepo>,
    item_repo: Arc<dyn traits::ItemRepo>,
    push_token_repo: Arc<dyn traits::PushTokenRepo>,
    redis: redis::Client,
    http: reqwest::Client,
}

impl PgReminderService {
    pub fn new(
        user_repo: Arc<dyn traits::UserRepo>,
        item_repo: Arc<dyn traits::ItemRepo>,
        push_token_repo: Arc<dyn traits::PushTokenRepo>,
        redis: redis::Client,
        http: reqwest::Client,
    ) -> Self {
        Self {
            user_repo,
            item_repo,
            push_token_repo,
            redis,
            http,
        }
    }

    async fn process_user(&self, user_id: Uuid, freq: &str) {
        // 1. Anti-spam: SET NX EX
        let redis_key = format!("last_reminder:{user_id}");
        let ttl = freq_ttl(freq);

        let mut conn = match self.redis.get_multiplexed_async_connection().await {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(user_id = %user_id, error = %e, "redis connection failed, skipping");
                return;
            }
        };

        let set: bool = match redis::cmd("SET")
            .arg(&redis_key)
            .arg("1")
            .arg("NX")
            .arg("EX")
            .arg(ttl)
            .query_async(&mut conn)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(user_id = %user_id, error = %e, "redis SET NX failed, skipping");
                return;
            }
        };

        if !set {
            tracing::debug!(user_id = %user_id, "anti-spam: reminder already sent recently");
            return;
        }

        // 2. Find eligible items (active, older than MIN_AGE_DAYS)
        let cutoff = Utc::now() - TimeDelta::try_days(MIN_AGE_DAYS).expect("valid days");
        let items = match self.item_repo.find_active_older_than(user_id, cutoff).await {
            Ok(items) => items,
            Err(e) => {
                tracing::error!(user_id = %user_id, error = %e, "failed to fetch items");
                // Delete anti-spam key since we couldn't process
                let _: Result<(), _> = conn.del(&redis_key).await;
                return;
            }
        };

        if items.is_empty() {
            // No eligible items — don't penalize the user
            let _: Result<(), _> = conn.del(&redis_key).await;
            tracing::debug!(user_id = %user_id, "no eligible items for reminder");
            return;
        }

        // 3. Score and select top N items
        let mut scored: Vec<(f64, &crate::models::Item)> = items
            .iter()
            .map(|item| {
                let days = (Utc::now() - item.created_at).num_days();
                (score(item.priority, days), item)
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let top_items: Vec<&crate::models::Item> = scored
            .into_iter()
            .take(TOP_N)
            .map(|(_, item)| item)
            .collect();

        // 4. Get push tokens
        let tokens = match self.push_token_repo.find_by_user(user_id).await {
            Ok(t) => t,
            Err(e) => {
                tracing::error!(user_id = %user_id, error = %e, "failed to fetch push tokens");
                return;
            }
        };

        if tokens.is_empty() {
            tracing::debug!(user_id = %user_id, "no push tokens registered");
            return;
        }

        // 5. Build notification
        let title = "Tes envies t'attendent !".to_string();
        let body = if top_items.len() == 1 {
            format!("N'oublie pas : {}", top_items[0].name)
        } else {
            let names: Vec<&str> = top_items.iter().map(|i| i.name.as_str()).collect();
            format!("N'oublie pas : {}", names.join(", "))
        };

        // 6. Send to all tokens (batch of 100)
        let messages: Vec<ExpoPushMessage> = tokens
            .iter()
            .map(|pt| ExpoPushMessage {
                to: pt.token.clone(),
                title: title.clone(),
                body: body.clone(),
                sound: "default",
            })
            .collect();

        for chunk in messages.chunks(100) {
            match self
                .http
                .post("https://exp.host/--/api/v2/push/send")
                .json(&chunk)
                .send()
                .await
            {
                Ok(resp) => {
                    if let Ok(push_resp) = resp.json::<ExpoPushResponse>().await {
                        // Handle DeviceNotRegistered — remove invalid tokens
                        for (i, ticket) in push_resp.data.iter().enumerate() {
                            if ticket.status == "error"
                                && let Some(details) = &ticket.details
                                && details.error.as_deref() == Some("DeviceNotRegistered")
                                && let Some(msg) = chunk.get(i)
                            {
                                tracing::info!(
                                    token = %msg.to,
                                    "removing unregistered device token"
                                );
                                let _ =
                                    self.push_token_repo.delete_by_token(user_id, &msg.to).await;
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(user_id = %user_id, error = %e, "expo push send failed");
                }
            }
        }

        tracing::info!(
            user_id = %user_id,
            items_count = top_items.len(),
            tokens_count = tokens.len(),
            "reminder sent"
        );
    }
}

#[async_trait]
impl traits::ReminderService for PgReminderService {
    async fn execute_hourly_tick(&self) {
        let current_hour = Utc::now().hour() as i16;

        let users = match self
            .user_repo
            .find_eligible_for_reminder(current_hour)
            .await
        {
            Ok(u) => u,
            Err(e) => {
                tracing::error!(error = %e, "failed to fetch eligible users for reminder");
                return;
            }
        };

        tracing::info!(
            hour = current_hour,
            eligible_users = users.len(),
            "reminder tick started"
        );

        for user in &users {
            self.process_user(user.id, &user.reminder_freq).await;
        }

        tracing::info!(hour = current_hour, "reminder tick completed");
    }
}

use chrono::Timelike;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_priority_scores_higher() {
        // priority 1 (highest user priority) vs priority 3 (lowest), same age
        let score_high = score(1, 10);
        let score_low = score(3, 10);
        assert!(
            score_high > score_low,
            "priority 1 ({score_high}) should score higher than priority 3 ({score_low})"
        );
    }

    #[test]
    fn older_items_score_higher() {
        // Same priority, different ages
        let score_old = score(2, 30);
        let score_new = score(2, 8);
        assert!(
            score_old > score_new,
            "30 days ({score_old}) should score higher than 8 days ({score_new})"
        );
    }

    #[test]
    fn freq_ttl_values() {
        assert_eq!(freq_ttl("daily"), 82_800);
        assert_eq!(freq_ttl("weekly"), 561_600);
        assert_eq!(freq_ttl("monthly"), 2_505_600);
    }

    #[test]
    fn freq_ttl_unknown_falls_back_to_daily() {
        assert_eq!(freq_ttl("unknown"), 82_800);
    }

    #[test]
    fn age_can_compensate_lower_priority() {
        // priority 3 (low) with 60 days should beat priority 1 (high) with 7 days
        let score_low_prio_old = score(3, 60);
        let score_high_prio_new = score(1, 7);
        assert!(
            score_low_prio_old > score_high_prio_new,
            "p3+60d ({score_low_prio_old}) should beat p1+7d ({score_high_prio_new})"
        );
    }

    #[test]
    fn score_boundary_values() {
        // priority 1 (highest), minimum age
        let s = score(1, MIN_AGE_DAYS);
        assert!(s > 0.0, "score at boundary should be positive: {s}");

        // priority 3 (lowest), minimum age
        let s_low = score(3, MIN_AGE_DAYS);
        assert!(
            s > s_low,
            "p1 ({s}) should still beat p3 ({s_low}) at same age"
        );
    }
}
