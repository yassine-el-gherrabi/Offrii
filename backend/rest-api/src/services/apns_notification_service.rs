use std::io::Cursor;

use a2::request::notification::DefaultNotificationBuilder;
use a2::request::notification::NotificationBuilder;
use a2::request::notification::NotificationOptions;
use a2::response::ErrorReason;
use a2::{Client, ClientConfig, Endpoint};
use async_trait::async_trait;
use futures::stream::{self, StreamExt};

use crate::traits::{NotificationOutcome, NotificationRequest, NotificationService};

/// Max concurrent APNs requests per batch.
const MAX_CONCURRENT_SENDS: usize = 10;

pub struct ApnsNotificationService {
    client: Client,
    bundle_id: String,
}

impl ApnsNotificationService {
    pub fn new(
        key_pem: &[u8],
        key_id: &str,
        team_id: &str,
        bundle_id: &str,
        sandbox: bool,
    ) -> anyhow::Result<Self> {
        let endpoint = if sandbox {
            Endpoint::Sandbox
        } else {
            Endpoint::Production
        };

        let config = ClientConfig::new(endpoint);

        let client = Client::token(&mut Cursor::new(key_pem), key_id, team_id, config)
            .map_err(|e| anyhow::anyhow!("Failed to create APNs client: {e}"))?;

        Ok(Self {
            client,
            bundle_id: bundle_id.to_string(),
        })
    }

    async fn send_one(&self, msg: &NotificationRequest) -> NotificationOutcome {
        let builder = DefaultNotificationBuilder::new()
            .set_title(&msg.title)
            .set_body(&msg.body)
            .set_sound("default");

        let options = NotificationOptions {
            apns_topic: Some(&self.bundle_id),
            ..Default::default()
        };

        let payload = builder.build(&msg.device_token, options);

        match self.client.send(payload).await {
            Ok(resp) if resp.code == 200 => NotificationOutcome::Sent,
            Ok(resp) => {
                if let Some(body) = resp.error {
                    match body.reason {
                        ErrorReason::BadDeviceToken | ErrorReason::Unregistered => {
                            tracing::info!(
                                token = %redact_token(&msg.device_token),
                                reason = ?body.reason,
                                "invalid device token"
                            );
                            NotificationOutcome::InvalidToken
                        }
                        other => {
                            tracing::warn!(
                                token = %redact_token(&msg.device_token),
                                reason = ?other,
                                "APNs error"
                            );
                            NotificationOutcome::Error(format!("{other:?}"))
                        }
                    }
                } else {
                    NotificationOutcome::Error(format!(
                        "APNs returned status {} with no error body",
                        resp.code
                    ))
                }
            }
            Err(e) => {
                tracing::error!(
                    token = %redact_token(&msg.device_token),
                    error = %e,
                    "APNs send failed"
                );
                NotificationOutcome::Error(e.to_string())
            }
        }
    }
}

#[async_trait]
impl NotificationService for ApnsNotificationService {
    async fn send_batch(&self, messages: &[NotificationRequest]) -> Vec<NotificationOutcome> {
        let futures: Vec<_> = messages.iter().map(|msg| self.send_one(msg)).collect();
        stream::iter(futures)
            .buffer_unordered(MAX_CONCURRENT_SENDS)
            .collect()
            .await
    }
}

/// Redact a device token for logging: show first 8 and last 4 chars.
fn redact_token(token: &str) -> String {
    if token.len() > 12 {
        format!("{}…{}", &token[..8], &token[token.len() - 4..])
    } else {
        "***".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_fails_with_invalid_pem() {
        let result =
            ApnsNotificationService::new(b"not a valid pem", "KEY", "TEAM", "com.test", true);
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(
            err.to_string().contains("Failed to create APNs client"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn new_fails_with_empty_key() {
        let result = ApnsNotificationService::new(b"", "KEY", "TEAM", "com.test", false);
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(
            err.to_string().contains("Failed to create APNs client"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn redact_token_long() {
        let token = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6abcd";
        assert_eq!(redact_token(token), "a1b2c3d4…abcd");
    }

    #[test]
    fn redact_token_short() {
        assert_eq!(redact_token("abc"), "***");
    }
}
