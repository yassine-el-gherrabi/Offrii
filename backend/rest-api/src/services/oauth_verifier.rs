use std::sync::Arc;
use std::time::{Duration, Instant};

use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode_header};
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::errors::AppError;

const JWKS_CACHE_TTL: Duration = Duration::from_secs(3600);
const GOOGLE_JWKS_URL: &str = "https://www.googleapis.com/oauth2/v3/certs";
const APPLE_JWKS_URL: &str = "https://appleid.apple.com/auth/keys";

/// Claims extracted from a verified OAuth ID token.
#[derive(Debug, Clone)]
pub struct OAuthClaims {
    pub sub: String,
    pub email: String,
    pub name: Option<String>,
}

/// JSON Web Key as returned by Google/Apple JWKS endpoints.
#[derive(Debug, Deserialize)]
struct Jwk {
    kid: String,
    kty: String,
    n: String,
    e: String,
}

#[derive(Debug, Deserialize)]
struct JwkSet {
    keys: Vec<Jwk>,
}

/// Google ID token claims.
#[derive(Debug, Deserialize)]
#[allow(dead_code)] // iss/aud validated by jsonwebtoken::Validation, not accessed directly
struct GoogleClaims {
    sub: String,
    email: String,
    email_verified: Option<bool>,
    name: Option<String>,
    iss: String,
    aud: String,
}

/// Apple ID token claims.
#[derive(Debug, Deserialize)]
#[allow(dead_code)] // iss/aud validated by jsonwebtoken::Validation, not accessed directly
struct AppleClaims {
    sub: String,
    email: Option<String>,
    iss: String,
    aud: String,
}

/// Custom validation that skips exp check (we do manual checks as needed).
/// Actually, we DO want exp validation — `jsonwebtoken` handles that automatically.
fn make_validation(aud: &str, issuers: &[&str]) -> Validation {
    let mut v = Validation::new(Algorithm::RS256);
    v.set_audience(&[aud]);
    v.set_issuer(issuers);
    v
}

pub struct OAuthVerifier {
    http: reqwest::Client,
    google_client_id: Option<String>,
    apple_bundle_id: String,
    google_jwks: Arc<RwLock<Option<(Instant, JwkSet)>>>,
    apple_jwks: Arc<RwLock<Option<(Instant, JwkSet)>>>,
}

impl OAuthVerifier {
    pub fn new(google_client_id: Option<String>, apple_bundle_id: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            google_client_id,
            apple_bundle_id,
            google_jwks: Arc::new(RwLock::new(None)),
            apple_jwks: Arc::new(RwLock::new(None)),
        }
    }

    async fn fetch_jwks_cached(
        &self,
        cache: &RwLock<Option<(Instant, JwkSet)>>,
        url: &str,
    ) -> Result<(), AppError> {
        // Check if cached and still valid
        {
            let guard = cache.read().await;
            if let Some((fetched_at, _)) = guard.as_ref()
                && fetched_at.elapsed() < JWKS_CACHE_TTL
            {
                return Ok(());
            }
        }
        // Fetch fresh JWKS
        let resp = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("JWKS fetch failed: {e}")))?;

        let jwks: JwkSet = resp
            .json()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("JWKS parse failed: {e}")))?;

        let mut guard = cache.write().await;
        *guard = Some((Instant::now(), jwks));
        Ok(())
    }

    fn find_decoding_key(jwks: &JwkSet, kid: &str) -> Result<DecodingKey, AppError> {
        let jwk = jwks
            .keys
            .iter()
            .find(|k| k.kid == kid && k.kty == "RSA")
            .ok_or_else(|| {
                AppError::Unauthorized(format!("no matching JWK found for kid={kid}"))
            })?;

        DecodingKey::from_rsa_components(&jwk.n, &jwk.e).map_err(|e| {
            AppError::Internal(anyhow::anyhow!("failed to build RSA decoding key: {e}"))
        })
    }

    /// Verify a Google ID token and return the extracted claims.
    pub async fn verify_google(&self, id_token: &str) -> Result<OAuthClaims, AppError> {
        let client_id = self
            .google_client_id
            .as_ref()
            .ok_or_else(|| AppError::BadRequest("Google Sign-In is not configured".into()))?;

        self.fetch_jwks_cached(&self.google_jwks, GOOGLE_JWKS_URL)
            .await?;

        let header = decode_header(id_token)
            .map_err(|e| AppError::Unauthorized(format!("invalid token header: {e}")))?;

        let kid = header
            .kid
            .ok_or_else(|| AppError::Unauthorized("token header missing kid".into()))?;

        let guard = self.google_jwks.read().await;
        let jwks = &guard.as_ref().unwrap().1;
        let key = Self::find_decoding_key(jwks, &kid)?;

        let validation = make_validation(
            client_id,
            &["accounts.google.com", "https://accounts.google.com"],
        );

        let token_data = jsonwebtoken::decode::<GoogleClaims>(id_token, &key, &validation)
            .map_err(|e| {
                AppError::Unauthorized(format!("Google token verification failed: {e}"))
            })?;

        let claims = token_data.claims;

        if claims.email_verified != Some(true) {
            return Err(AppError::Unauthorized("Google email not verified".into()));
        }

        Ok(OAuthClaims {
            sub: claims.sub,
            email: claims.email,
            name: claims.name,
        })
    }

    /// Verify an Apple ID token and return the extracted claims.
    pub async fn verify_apple(&self, id_token: &str) -> Result<OAuthClaims, AppError> {
        self.fetch_jwks_cached(&self.apple_jwks, APPLE_JWKS_URL)
            .await?;

        let header = decode_header(id_token)
            .map_err(|e| AppError::Unauthorized(format!("invalid token header: {e}")))?;

        let kid = header
            .kid
            .ok_or_else(|| AppError::Unauthorized("token header missing kid".into()))?;

        let guard = self.apple_jwks.read().await;
        let jwks = &guard.as_ref().unwrap().1;
        let key = Self::find_decoding_key(jwks, &kid)?;

        let validation = make_validation(&self.apple_bundle_id, &["https://appleid.apple.com"]);

        let token_data = jsonwebtoken::decode::<AppleClaims>(id_token, &key, &validation)
            .map_err(|e| AppError::Unauthorized(format!("Apple token verification failed: {e}")))?;

        let claims = token_data.claims;

        Ok(OAuthClaims {
            sub: claims.sub,
            email: claims.email.unwrap_or_default(),
            name: None, // Apple doesn't include name in the JWT
        })
    }
}
