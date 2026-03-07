use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::dto::push_tokens::PushTokenResponse;
use crate::errors::AppError;
use crate::traits;

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgPushTokenService {
    push_token_repo: Arc<dyn traits::PushTokenRepo>,
}

impl PgPushTokenService {
    pub fn new(push_token_repo: Arc<dyn traits::PushTokenRepo>) -> Self {
        Self { push_token_repo }
    }
}

#[async_trait]
impl traits::PushTokenService for PgPushTokenService {
    async fn register_token(
        &self,
        user_id: Uuid,
        token: &str,
        platform: &str,
    ) -> Result<PushTokenResponse, AppError> {
        let pt = self
            .push_token_repo
            .upsert(user_id, token, platform)
            .await
            .map_err(AppError::Internal)?;

        Ok(PushTokenResponse::from(&pt))
    }

    async fn unregister_token(&self, user_id: Uuid, token: &str) -> Result<(), AppError> {
        let deleted = self
            .push_token_repo
            .delete_by_token(user_id, token)
            .await
            .map_err(AppError::Internal)?;

        if !deleted {
            return Err(AppError::NotFound("push token not found".into()));
        }

        Ok(())
    }
}
