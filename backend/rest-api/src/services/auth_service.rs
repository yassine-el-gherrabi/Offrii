use std::sync::{Arc, LazyLock};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use async_trait::async_trait;
use chrono::{TimeDelta, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::auth::{AuthResponse, RefreshResponse, TokenPair, UserResponse};
use crate::errors::AppError;
use crate::models::User;
use crate::repositories::{refresh_token_repo, user_repo};
use crate::services::oauth_verifier::OAuthVerifier;
use crate::traits;
use crate::utils::hash;
use crate::utils::jwt::{ACCESS_TOKEN_TTL_SECS, JwtKeys, REFRESH_TOKEN_TTL_SECS};
use crate::utils::password_policy::{self, PasswordPolicyViolation};
use crate::utils::token_hash::sha256_hex;
use crate::utils::username::{is_reserved_username, is_valid_username};

/// Generate a cryptographically random 64-character hex token for email verification.
fn generate_verification_token() -> String {
    let bytes: [u8; 32] = rand::random();
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Maximum number of active refresh tokens kept per user.
const MAX_REFRESH_TOKENS_PER_USER: i64 = 5;

/// Pre-computed Argon2id hash used as a timing side-channel countermeasure.
/// When a login attempt targets a non-existent email, we still run Argon2
/// verification against this dummy hash so the response time is
/// indistinguishable from a "wrong password" attempt.
static DUMMY_HASH: LazyLock<String> = LazyLock::new(|| {
    hash::hash_password("timing-safe-dummy").expect("failed to generate dummy hash")
});

fn generate_token_pair(jwt: &JwtKeys, user_id: Uuid, token_version: i32) -> Result<TokenPair> {
    let access_token = jwt.generate_access_token(user_id, token_version)?;
    let refresh_token = jwt.generate_refresh_token(user_id, token_version)?;
    Ok(TokenPair {
        access_token,
        refresh_token,
        token_type: "Bearer",
        expires_in: ACCESS_TOKEN_TTL_SECS,
    })
}

fn refresh_expires_at() -> chrono::DateTime<Utc> {
    Utc::now() + TimeDelta::try_seconds(REFRESH_TOKEN_TTL_SECS as i64).expect("valid refresh TTL")
}

// ── Concrete implementation ──────────────────────────────────────────

pub struct PgAuthService {
    /// Held for transactional operations (`register`, `refresh`).
    pool: PgPool,
    user_repo: Arc<dyn traits::UserRepo>,
    refresh_token_repo: Arc<dyn traits::RefreshTokenRepo>,
    jwt: Arc<JwtKeys>,
    redis: redis::Client,
    email_service: Arc<dyn traits::EmailService>,
    oauth_verifier: Arc<OAuthVerifier>,
}

/// Check if a sqlx database error is a unique-constraint violation (PG code 23505).
fn is_unique_violation(err: &anyhow::Error) -> bool {
    err.downcast_ref::<sqlx::Error>()
        .and_then(|e| match e {
            sqlx::Error::Database(db_err) => db_err.code().map(|c| c == "23505"),
            _ => None,
        })
        .unwrap_or(false)
}

impl PgAuthService {
    pub fn new(
        pool: PgPool,
        user_repo: Arc<dyn traits::UserRepo>,
        refresh_token_repo: Arc<dyn traits::RefreshTokenRepo>,
        jwt: Arc<JwtKeys>,
        redis: redis::Client,
        email_service: Arc<dyn traits::EmailService>,
        oauth_verifier: Arc<OAuthVerifier>,
    ) -> Self {
        Self {
            pool,
            user_repo,
            refresh_token_repo,
            jwt,
            redis,
            email_service,
            oauth_verifier,
        }
    }
}

#[async_trait]
impl traits::AuthService for PgAuthService {
    #[tracing::instrument(skip(self, password))]
    async fn register(
        &self,
        email: &str,
        password: &str,
        display_name: Option<&str>,
        requested_username: Option<&str>,
    ) -> Result<AuthResponse, AppError> {
        let email = email.trim().to_lowercase();

        // OWASP password policy: length, common passwords, breach check
        password_policy::check(password)
            .await
            .map_err(|v| match v {
                PasswordPolicyViolation::Common => AppError::BadRequest("password_common".into()),
                PasswordPolicyViolation::Breached => {
                    AppError::BadRequest("password_breached".into())
                }
                PasswordPolicyViolation::TooShort | PasswordPolicyViolation::TooLong => {
                    AppError::BadRequest("password_length".into())
                }
            })?;

        // Hash password on blocking thread (CPU-bound)
        let password_owned = password.to_string();
        let password_hash =
            tokio::task::spawn_blocking(move || hash::hash_password(&password_owned))
                .await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("spawn_blocking join error: {e}")))?
                .map_err(AppError::Internal)?;

        // Resolve username: use provided value (with validation) or auto-generate
        let username = if let Some(uname) = requested_username {
            if !is_valid_username(uname) {
                return Err(AppError::BadRequest(
                    "username must be 3-30 characters, start with a letter, and contain only lowercase letters, digits, and underscores".into(),
                ));
            }
            if is_reserved_username(uname) {
                return Err(AppError::BadRequest("this username is reserved".into()));
            }
            let taken: bool =
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)")
                    .bind(uname)
                    .fetch_one(&self.pool)
                    .await
                    .map_err(|e| AppError::Internal(e.into()))?;
            if taken {
                return Err(AppError::Conflict("username already taken".into()));
            }
            uname.to_string()
        } else {
            let base: &str = match display_name {
                Some(name) => name,
                None => email.split('@').next().unwrap_or("user"),
            };
            generate_unique_username(base, &self.pool).await?
        };

        // Check if email exists and provide a helpful error for OAuth accounts
        if let Some(existing) = self
            .user_repo
            .find_by_email(&email)
            .await
            .map_err(AppError::Internal)?
        {
            if let Some(ref provider) = existing.oauth_provider {
                return Err(AppError::Conflict(format!("email_uses_oauth:{provider}")));
            }
            return Err(AppError::Conflict("email already registered".into()));
        }

        // Transaction: create user + copy categories + insert refresh token
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        let user =
            user_repo::create_user(&mut *tx, &email, &username, &password_hash, display_name)
                .await
                .map_err(|e| {
                    if is_unique_violation(&e) {
                        let msg = format!("{e}");
                        if msg.contains("users_username_unique") || msg.contains("username") {
                            AppError::Internal(anyhow::anyhow!(
                                "username collision during registration"
                            ))
                        } else {
                            AppError::Conflict("email already registered".into())
                        }
                    } else {
                        AppError::Internal(e)
                    }
                })?;

        let tokens = generate_token_pair(&self.jwt, user.id, user.token_version)
            .map_err(AppError::Internal)?;
        let refresh_hash = sha256_hex(&tokens.refresh_token);

        refresh_token_repo::insert(&mut *tx, user.id, &refresh_hash, refresh_expires_at())
            .await
            .map_err(AppError::Internal)?;

        tx.commit()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        // Send welcome email asynchronously (non-blocking, fire-and-forget)
        let email_svc = self.email_service.clone();
        let to = email.clone();
        let name = display_name.map(|s| s.to_string());
        tokio::spawn(async move {
            if let Err(e) = email_svc.send_welcome_email(&to, name.as_deref()).await {
                tracing::error!("failed to send welcome email: {e}");
            }
        });

        // Generate email verification token and send verification email
        let verification_token = generate_verification_token();
        sqlx::query("INSERT INTO email_verification_tokens (user_id, token) VALUES ($1, $2)")
            .bind(user.id)
            .bind(&verification_token)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        let email_svc = self.email_service.clone();
        let to = email.clone();
        let token = verification_token.clone();
        tokio::spawn(async move {
            if let Err(e) = email_svc.send_verification_email(&to, &token).await {
                tracing::error!("failed to send verification email: {e}");
            }
        });

        Ok(AuthResponse {
            tokens,
            user: UserResponse::from(&user),
            is_new_user: true,
        })
    }

    #[tracing::instrument(skip(self, password))]
    async fn login(&self, identifier: &str, password: &str) -> Result<AuthResponse, AppError> {
        let identifier = identifier.trim().to_lowercase();
        let invalid_credentials =
            || AppError::Unauthorized("invalid email/username or password".into());

        // Try email first, then username
        let user: Option<User> = if identifier.contains('@') {
            self.user_repo
                .find_by_email(&identifier)
                .await
                .map_err(AppError::Internal)?
        } else {
            self.user_repo
                .find_by_username(&identifier)
                .await
                .map_err(AppError::Internal)?
        };

        // Always run Argon2 verify — even when user doesn't exist — to prevent
        // timing side-channel attacks that reveal whether an email is registered.
        let password_hash_to_check = match &user {
            Some(u) => u
                .password_hash
                .clone()
                .unwrap_or_else(|| DUMMY_HASH.clone()),
            None => DUMMY_HASH.clone(),
        };

        let password_owned = password.to_string();
        let valid = tokio::task::spawn_blocking(move || {
            hash::verify_password(&password_owned, &password_hash_to_check)
        })
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("spawn_blocking join error: {e}")))?
        .map_err(AppError::Internal)?;

        let user = match (valid, user) {
            (true, Some(u)) => u,
            _ => return Err(invalid_credentials()),
        };

        let tokens = generate_token_pair(&self.jwt, user.id, user.token_version)
            .map_err(AppError::Internal)?;
        let refresh_hash = sha256_hex(&tokens.refresh_token);

        self.refresh_token_repo
            .insert(user.id, &refresh_hash, refresh_expires_at())
            .await
            .map_err(AppError::Internal)?;

        // Enforce maximum active refresh tokens per user
        self.refresh_token_repo
            .revoke_excess_for_user(user.id, MAX_REFRESH_TOKENS_PER_USER)
            .await
            .map_err(AppError::Internal)?;

        Ok(AuthResponse {
            tokens,
            user: UserResponse::from(&user),
            is_new_user: false,
        })
    }

    #[tracing::instrument(skip(self, raw_refresh_token))]
    async fn refresh(&self, raw_refresh_token: &str) -> Result<RefreshResponse, AppError> {
        let claims = self
            .jwt
            .validate_refresh_token(raw_refresh_token)
            .map_err(|_| AppError::Unauthorized("invalid or expired refresh token".into()))?;

        let user_id: Uuid = claims
            .sub
            .parse()
            .map_err(|_| AppError::Unauthorized("invalid token subject".into()))?;

        // Fetch user for current token_version (defense-in-depth)
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::Unauthorized("user not found".into()))?;

        if claims.token_version < user.token_version {
            return Err(AppError::Unauthorized("token version revoked".into()));
        }

        let old_hash = sha256_hex(raw_refresh_token);

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        // Atomic revoke (UPDATE acquires row lock, prevents concurrent refresh)
        let revoked = refresh_token_repo::revoke_by_hash_for_user(&mut *tx, &old_hash, user_id)
            .await
            .map_err(AppError::Internal)?;

        if !revoked {
            return Err(AppError::Unauthorized(
                "refresh token revoked or not found".into(),
            ));
        }

        let tokens = generate_token_pair(&self.jwt, user_id, user.token_version)
            .map_err(AppError::Internal)?;
        let new_hash = sha256_hex(&tokens.refresh_token);

        refresh_token_repo::insert(&mut *tx, user_id, &new_hash, refresh_expires_at())
            .await
            .map_err(AppError::Internal)?;

        tx.commit()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        Ok(RefreshResponse { tokens })
    }

    #[tracing::instrument(skip(self))]
    async fn logout(&self, user_id: Uuid, jti: &str, token_exp: usize) -> Result<(), AppError> {
        // 1. Revoke all refresh tokens (existing behaviour)
        self.refresh_token_repo
            .revoke_all_for_user(user_id)
            .await
            .map_err(AppError::Internal)?;

        // 2. Blacklist the access-token JTI in Redis for its remaining TTL
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as usize;
        let remaining_ttl = token_exp
            .saturating_sub(now)
            .min(ACCESS_TOKEN_TTL_SECS as usize);

        if remaining_ttl > 0 {
            if let Ok(mut conn) = self.redis.get_multiplexed_async_connection().await {
                let key = format!("blacklist:{jti}");
                let _: Result<(), _> = redis::cmd("SET")
                    .arg(&key)
                    .arg("1")
                    .arg("EX")
                    .arg(remaining_ttl)
                    .arg("NX")
                    .query_async(&mut conn)
                    .await;
            } else {
                tracing::warn!("redis unavailable – skipping JTI blacklist on logout");
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip(self, current_password, new_password))]
    async fn change_password(
        &self,
        user_id: Uuid,
        current_password: &str,
        new_password: &str,
    ) -> Result<(), AppError> {
        // 1. Fetch user
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("user not found".into()))?;

        // 2. Reject if OAuth-only user (no password set)
        let hash = match user.password_hash.clone() {
            Some(h) => h,
            None => {
                return Err(AppError::BadRequest(
                    "no password set for this account".into(),
                ));
            }
        };

        // 3. Verify current password on blocking thread
        let current_owned = current_password.to_string();
        let valid =
            tokio::task::spawn_blocking(move || hash::verify_password(&current_owned, &hash))
                .await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("spawn_blocking join error: {e}")))?
                .map_err(AppError::Internal)?;

        if !valid {
            return Err(AppError::Unauthorized("invalid current password".into()));
        }

        // 3. Enforce password policy on new password
        password_policy::check(new_password)
            .await
            .map_err(|v| match v {
                PasswordPolicyViolation::Common => AppError::BadRequest("password_common".into()),
                PasswordPolicyViolation::Breached => {
                    AppError::BadRequest("password_breached".into())
                }
                PasswordPolicyViolation::TooShort | PasswordPolicyViolation::TooLong => {
                    AppError::BadRequest("password_length".into())
                }
            })?;

        // 4. Hash new password on blocking thread
        let new_owned = new_password.to_string();
        let new_hash = tokio::task::spawn_blocking(move || hash::hash_password(&new_owned))
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("spawn_blocking join error: {e}")))?
            .map_err(AppError::Internal)?;

        // 5. Persist new hash
        let updated = self
            .user_repo
            .update_password_hash(user_id, &new_hash)
            .await
            .map_err(AppError::Internal)?;

        if !updated {
            return Err(AppError::NotFound("user not found".into()));
        }

        // 5. Invalidate all tokens (force re-login everywhere)
        self.invalidate_all_tokens(user_id).await?;

        // 6. Notify user (fire-and-forget)
        let email_svc = self.email_service.clone();
        let to = user.email.clone();
        tokio::spawn(async move {
            if let Err(e) = email_svc.send_password_changed_email(&to).await {
                tracing::warn!(error = %e, "failed to send password changed email");
            }
        });

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn forgot_password(&self, email: &str) -> Result<(), AppError> {
        let email = email.trim().to_lowercase();

        // Rate limit: one request per email per 60 seconds
        if let Ok(mut conn) = self.redis.get_multiplexed_async_connection().await {
            let rate_key = format!("pwreset:rate:{email}");
            let set: Result<bool, _> = redis::cmd("SET")
                .arg(&rate_key)
                .arg("1")
                .arg("EX")
                .arg(60)
                .arg("NX")
                .query_async(&mut conn)
                .await;

            // If SET NX failed (key already exists), silently return Ok
            if set.is_err() || !set.unwrap_or(false) {
                return Ok(());
            }

            // Resend rate limit: max 3 per 5 minutes
            let resend_5m_key = format!("pwreset:resend_5m:{email}");
            let resend_5m: i64 = redis::cmd("INCR")
                .arg(&resend_5m_key)
                .query_async(&mut conn)
                .await
                .unwrap_or(1);
            if resend_5m == 1 {
                let _: Result<(), _> = redis::cmd("EXPIRE")
                    .arg(&resend_5m_key)
                    .arg(300)
                    .query_async(&mut conn)
                    .await;
            }
            if resend_5m > 3 {
                return Ok(());
            }

            // Resend rate limit: max 10 per day
            let resend_daily_key = format!("pwreset:resend_daily:{email}");
            let resend_daily: i64 = redis::cmd("INCR")
                .arg(&resend_daily_key)
                .query_async(&mut conn)
                .await
                .unwrap_or(1);
            if resend_daily == 1 {
                let _: Result<(), _> = redis::cmd("EXPIRE")
                    .arg(&resend_daily_key)
                    .arg(86400)
                    .query_async(&mut conn)
                    .await;
            }
            if resend_daily > 10 {
                return Ok(());
            }
        }

        // Look up user — if not found, return Ok silently (no email enumeration)
        let user = self
            .user_repo
            .find_by_email(&email)
            .await
            .map_err(AppError::Internal)?;

        let Some(_user) = user else {
            return Ok(());
        };

        // Generate 6-digit code
        let n: u32 = rand::random_range(0..1_000_000);
        let code = format!("{n:06}");

        // Hash the code and store in Redis with 30-minute TTL
        let code_hash = sha256_hex(&code);
        let stored = if let Ok(mut conn) = self.redis.get_multiplexed_async_connection().await {
            let key = format!("pwreset:{email}");
            redis::cmd("SET")
                .arg(&key)
                .arg(&code_hash)
                .arg("EX")
                .arg(1800) // 30 minutes
                .query_async::<()>(&mut conn)
                .await
                .is_ok()
        } else {
            false
        };

        // Only send email if code was stored successfully
        if stored {
            let email_svc = self.email_service.clone();
            let to = email.clone();
            let code_clone = code.clone();
            tokio::spawn(async move {
                if let Err(e) = email_svc.send_password_reset_code(&to, &code_clone).await {
                    tracing::error!("failed to send password reset email: {e}");
                }
            });
        } else {
            tracing::error!("failed to store password reset code in redis, skipping email");
        }

        Ok(())
    }

    #[tracing::instrument(skip(self, code))]
    async fn verify_reset_code(&self, email: &str, code: &str) -> Result<(), AppError> {
        let email = email.trim().to_lowercase();

        let mut conn = self
            .redis
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "redis unavailable for verify reset code");
                AppError::Internal(anyhow::anyhow!("service unavailable"))
            })?;

        // Brute-force protection: same attempt counter as reset_password
        let attempt_key = format!("pwreset:attempts:{email}");
        let attempts: i64 = redis::cmd("INCR")
            .arg(&attempt_key)
            .query_async(&mut conn)
            .await
            .unwrap_or(1);
        if attempts == 1 {
            let _: Result<(), _> = redis::cmd("EXPIRE")
                .arg(&attempt_key)
                .arg(1800)
                .query_async(&mut conn)
                .await;
        }
        if attempts > 5 {
            let _: Result<(), _> = redis::cmd("DEL")
                .arg(format!("pwreset:{email}"))
                .arg(&attempt_key)
                .arg(format!("pwreset:rate:{email}"))
                .query_async(&mut conn)
                .await;
            return Err(AppError::BadRequest("too_many_attempts".into()));
        }

        let stored_hash: Option<String> = {
            let key = format!("pwreset:{email}");
            redis::cmd("GET")
                .arg(&key)
                .query_async(&mut conn)
                .await
                .ok()
        };

        let stored_hash =
            stored_hash.ok_or_else(|| AppError::BadRequest("invalid_or_expired_code".into()))?;

        // Verify code hash — do NOT delete the code (it will be consumed by reset_password)
        let code_hash = sha256_hex(code);
        if code_hash != stored_hash {
            return Err(AppError::BadRequest("invalid_or_expired_code".into()));
        }

        Ok(())
    }

    #[tracing::instrument(skip(self, code, new_password))]
    async fn reset_password(
        &self,
        email: &str,
        code: &str,
        new_password: &str,
    ) -> Result<(), AppError> {
        let email = email.trim().to_lowercase();

        // Read stored hash from Redis
        let mut conn = self
            .redis
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "redis unavailable for password reset");
                AppError::Internal(anyhow::anyhow!("service unavailable"))
            })?;

        // Brute-force protection: limit verification attempts
        let attempt_key = format!("pwreset:attempts:{email}");
        let attempts: i64 = redis::cmd("INCR")
            .arg(&attempt_key)
            .query_async(&mut conn)
            .await
            .unwrap_or(1);
        if attempts == 1 {
            let _: Result<(), _> = redis::cmd("EXPIRE")
                .arg(&attempt_key)
                .arg(1800)
                .query_async(&mut conn)
                .await;
        }
        if attempts > 5 {
            // Invalidate the reset code entirely
            let _: Result<(), _> = redis::cmd("DEL")
                .arg(format!("pwreset:{email}"))
                .arg(&attempt_key)
                .arg(format!("pwreset:rate:{email}"))
                .query_async(&mut conn)
                .await;
            return Err(AppError::BadRequest("too_many_attempts".into()));
        }

        let stored_hash: Option<String> = {
            let key = format!("pwreset:{email}");
            redis::cmd("GET")
                .arg(&key)
                .query_async(&mut conn)
                .await
                .ok()
        };

        let stored_hash =
            stored_hash.ok_or_else(|| AppError::BadRequest("invalid_or_expired_code".into()))?;

        // Verify code
        let code_hash = sha256_hex(code);
        if code_hash != stored_hash {
            return Err(AppError::BadRequest("invalid_or_expired_code".into()));
        }

        // Clean up Redis keys (including attempt counter)
        let _: Result<(), _> = redis::cmd("DEL")
            .arg(format!("pwreset:{email}"))
            .arg(format!("pwreset:rate:{email}"))
            .arg(&attempt_key)
            .query_async(&mut conn)
            .await;

        // Find user
        let user = self
            .user_repo
            .find_by_email(&email)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::BadRequest("invalid_or_expired_code".into()))?;

        // Enforce password policy on new password
        password_policy::check(new_password)
            .await
            .map_err(|v| match v {
                PasswordPolicyViolation::Common => AppError::BadRequest("password_common".into()),
                PasswordPolicyViolation::Breached => {
                    AppError::BadRequest("password_breached".into())
                }
                PasswordPolicyViolation::TooShort | PasswordPolicyViolation::TooLong => {
                    AppError::BadRequest("password_length".into())
                }
            })?;

        // Hash new password
        let new_owned = new_password.to_string();
        let new_hash = tokio::task::spawn_blocking(move || hash::hash_password(&new_owned))
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("spawn_blocking join error: {e}")))?
            .map_err(AppError::Internal)?;

        // Update password
        let updated = self
            .user_repo
            .update_password_hash(user.id, &new_hash)
            .await
            .map_err(AppError::Internal)?;

        if !updated {
            return Err(AppError::NotFound("user not found".into()));
        }

        // Invalidate all tokens
        self.invalidate_all_tokens(user.id).await?;

        // Notify user (fire-and-forget)
        let email_svc = self.email_service.clone();
        let to = user.email.clone();
        tokio::spawn(async move {
            if let Err(e) = email_svc.send_password_changed_email(&to).await {
                tracing::warn!(error = %e, "failed to send password changed email");
            }
        });

        Ok(())
    }

    #[tracing::instrument(skip(self, id_token))]
    async fn oauth_login(
        &self,
        provider: &str,
        id_token: &str,
        display_name: Option<&str>,
    ) -> Result<AuthResponse, AppError> {
        // 1. Verify the token according to provider
        let claims = match provider {
            "google" => self.oauth_verifier.verify_google(id_token).await?,
            "apple" => self.oauth_verifier.verify_apple(id_token).await?,
            _ => {
                return Err(AppError::BadRequest(format!(
                    "unsupported OAuth provider: {provider}"
                )));
            }
        };

        // 2. Check if user exists by OAuth provider ID
        let existing = self
            .user_repo
            .find_by_oauth(provider, &claims.sub)
            .await
            .map_err(AppError::Internal)?;

        let (user, is_new_user) = if let Some(mut user) = existing {
            // Existing OAuth user → login
            // Backfill avatar from Google if user has none
            if user.avatar_url.is_none()
                && let Some(ref picture) = claims.picture
            {
                let _ = sqlx::query("UPDATE users SET avatar_url = $1 WHERE id = $2")
                    .bind(picture.as_str())
                    .bind(user.id)
                    .execute(&self.pool)
                    .await;
                user.avatar_url = Some(picture.clone());
            }
            (user, false)
        } else {
            // 3. Check if email already exists → link OAuth
            let email_user = self
                .user_repo
                .find_by_email(&claims.email)
                .await
                .map_err(AppError::Internal)?;

            if let Some(user) = email_user.filter(|u| u.email_verified) {
                // Email is verified → safe to link OAuth provider
                self.user_repo
                    .link_oauth(user.id, provider, &claims.sub)
                    .await
                    .map_err(AppError::Internal)?;
                (user, false)
            } else {
                // 4. New user → create OAuth user
                let name = display_name.map(|s| s.to_string()).or(claims.name.clone());

                let username_base = name
                    .as_deref()
                    .unwrap_or(claims.email.split('@').next().unwrap_or("user"));
                let username = generate_unique_username(username_base, &self.pool).await?;

                let mut tx = self
                    .pool
                    .begin()
                    .await
                    .map_err(|e| AppError::Internal(e.into()))?;

                let user = user_repo::create_oauth_user(
                    &mut *tx,
                    &claims.email,
                    &username,
                    name.as_deref(),
                    provider,
                    &claims.sub,
                    claims.picture.as_deref(),
                )
                .await
                .map_err(|e| {
                    if is_unique_violation(&e) {
                        AppError::Conflict("email already registered".into())
                    } else {
                        AppError::Internal(e)
                    }
                })?;

                tx.commit()
                    .await
                    .map_err(|e| AppError::Internal(e.into()))?;

                // Send welcome email (fire-and-forget)
                let email_svc = self.email_service.clone();
                let to = claims.email.clone();
                let dn = name.clone();
                tokio::spawn(async move {
                    if let Err(e) = email_svc.send_welcome_email(&to, dn.as_deref()).await {
                        tracing::error!("failed to send welcome email: {e}");
                    }
                });

                (user, true)
            }
        };

        // 5. Generate token pair + store refresh token
        let tokens = generate_token_pair(&self.jwt, user.id, user.token_version)
            .map_err(AppError::Internal)?;
        let refresh_hash = sha256_hex(&tokens.refresh_token);

        self.refresh_token_repo
            .insert(user.id, &refresh_hash, refresh_expires_at())
            .await
            .map_err(AppError::Internal)?;

        self.refresh_token_repo
            .revoke_excess_for_user(user.id, MAX_REFRESH_TOKENS_PER_USER)
            .await
            .map_err(AppError::Internal)?;

        Ok(AuthResponse {
            tokens,
            user: UserResponse::from(&user),
            is_new_user,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn invalidate_all_tokens(&self, user_id: Uuid) -> Result<(), AppError> {
        // 1. Bump token_version in DB
        let new_version = self
            .user_repo
            .increment_token_version(user_id)
            .await
            .map_err(AppError::Internal)?;

        // 2. Broadcast new version via Redis (fail-open)
        if let Ok(mut conn) = self.redis.get_multiplexed_async_connection().await {
            let key = format!("tkver:{user_id}");
            let _: Result<(), _> = redis::cmd("SET")
                .arg(&key)
                .arg(new_version)
                .arg("EX")
                .arg(ACCESS_TOKEN_TTL_SECS)
                .query_async(&mut conn)
                .await;
        } else {
            tracing::warn!("redis unavailable – skipping tkver broadcast");
        }

        // 3. Revoke all refresh tokens
        self.refresh_token_repo
            .revoke_all_for_user(user_id)
            .await
            .map_err(AppError::Internal)?;

        Ok(())
    }

    #[tracing::instrument(skip(self, token))]
    async fn verify_email(&self, token: &str) -> Result<(), AppError> {
        // Look up the token and check expiry in a single query
        let row: Option<(Uuid, chrono::DateTime<Utc>)> = sqlx::query_as(
            "SELECT user_id, expires_at FROM email_verification_tokens WHERE token = $1",
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        let (user_id, expires_at) = row
            .ok_or_else(|| AppError::BadRequest("invalid or expired verification token".into()))?;

        if Utc::now() > expires_at {
            // Clean up expired token
            sqlx::query("DELETE FROM email_verification_tokens WHERE token = $1")
                .bind(token)
                .execute(&self.pool)
                .await
                .map_err(|e| AppError::Internal(e.into()))?;
            return Err(AppError::BadRequest(
                "invalid or expired verification token".into(),
            ));
        }

        // Set email_verified = true and delete the token atomically
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        sqlx::query("UPDATE users SET email_verified = true WHERE id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        sqlx::query("DELETE FROM email_verification_tokens WHERE token = $1")
            .bind(token)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        tx.commit()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn resend_verification(&self, user_id: Uuid) -> Result<(), AppError> {
        // Fetch user to check email_verified status
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound("user not found".into()))?;

        if user.email_verified {
            return Err(AppError::BadRequest("email already verified".into()));
        }

        // Rate limit: check if a token was created in the last 5 minutes
        let recent_token_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM email_verification_tokens \
             WHERE user_id = $1 AND created_at > NOW() - INTERVAL '5 minutes')",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        if recent_token_exists {
            return Err(AppError::TooManyRequests(
                "please wait before requesting another verification email".into(),
            ));
        }

        // Delete old tokens for this user
        sqlx::query("DELETE FROM email_verification_tokens WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        // Generate new token and insert
        let token = generate_verification_token();
        sqlx::query("INSERT INTO email_verification_tokens (user_id, token) VALUES ($1, $2)")
            .bind(user_id)
            .bind(&token)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        // Send verification email (fire-and-forget)
        let email_svc = self.email_service.clone();
        let to = user.email.clone();
        let token_clone = token.clone();
        tokio::spawn(async move {
            if let Err(e) = email_svc.send_verification_email(&to, &token_clone).await {
                tracing::error!("failed to send verification email: {e}");
            }
        });

        Ok(())
    }
}

// ── Username helpers ──────────────────────────────────────────────────

/// Slugify a string into lowercase alphanumeric characters only.
fn slugify(input: &str) -> String {
    input
        .chars()
        .filter_map(|c| {
            if c.is_ascii_alphanumeric() {
                Some(c.to_ascii_lowercase())
            } else {
                None
            }
        })
        .collect()
}

/// Generate a username candidate: slugified base + '_' + 4 random hex chars.
fn generate_username_candidate(base: &str) -> String {
    let slug = slugify(base);

    // Ensure slug starts with a letter; prepend 'u' if it starts with a digit or is empty
    let slug = if slug.is_empty() || slug.starts_with(|c: char| c.is_ascii_digit()) {
        format!("u{slug}")
    } else {
        slug
    };

    // Truncate to leave room for _xxxx suffix (5 chars)
    let max_base_len = 25;
    let truncated = if slug.len() > max_base_len {
        &slug[..max_base_len]
    } else {
        &slug
    };

    let suffix: u32 = rand::random_range(0..0x10000);
    format!("{truncated}_{suffix:04x}")
}

/// Generate a unique username, retrying up to 5 times if collisions occur.
async fn generate_unique_username(base: &str, pool: &PgPool) -> Result<String, AppError> {
    for _ in 0..5 {
        let candidate = generate_username_candidate(base);
        let taken: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)")
                .bind(&candidate)
                .fetch_one(pool)
                .await
                .map_err(|e| AppError::Internal(e.into()))?;

        if !taken {
            return Ok(candidate);
        }
    }

    // Extremely unlikely: 5 collisions in a row
    Err(AppError::Internal(anyhow::anyhow!(
        "failed to generate unique username after 5 attempts"
    )))
}
