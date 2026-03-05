use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Access token time-to-live: 15 minutes.
pub const ACCESS_TOKEN_TTL_SECS: u64 = 15 * 60;

/// Refresh token time-to-live: 7 days.
pub const REFRESH_TOKEN_TTL_SECS: u64 = 7 * 24 * 60 * 60;

/// RSA key size for dev-mode generation.
const RSA_KEY_BITS: usize = 2048;

/// Distinguishes access tokens from refresh tokens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

/// JWT claims payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub jti: String,
    pub token_type: TokenType,
}

/// Holds the RSA encoding (private) and decoding (public) keys for JWT operations.
pub struct JwtKeys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl JwtKeys {
    /// Build from PEM-encoded private and public key bytes.
    pub fn from_pem(private_pem: &[u8], public_pem: &[u8]) -> Result<Self> {
        let encoding = EncodingKey::from_rsa_pem(private_pem)
            .map_err(|e| anyhow::anyhow!("invalid RSA private key PEM: {e}"))?;
        let decoding = DecodingKey::from_rsa_pem(public_pem)
            .map_err(|e| anyhow::anyhow!("invalid RSA public key PEM: {e}"))?;
        Ok(Self { encoding, decoding })
    }

    /// Generate a fresh RSA 2048-bit key pair (dev mode).
    pub fn generate() -> Result<Self> {
        use rsa::RsaPrivateKey;
        use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};

        let mut rng = rand_core::OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, RSA_KEY_BITS)
            .map_err(|e| anyhow::anyhow!("failed to generate RSA key pair: {e}"))?;
        let public_key = private_key.to_public_key();

        let private_pem = private_key
            .to_pkcs8_pem(LineEnding::LF)
            .map_err(|e| anyhow::anyhow!("failed to encode private key to PEM: {e}"))?;
        let public_pem = public_key
            .to_public_key_pem(LineEnding::LF)
            .map_err(|e| anyhow::anyhow!("failed to encode public key to PEM: {e}"))?;

        Self::from_pem(private_pem.as_bytes(), public_pem.as_bytes())
    }

    /// Load keys from `JWT_PRIVATE_KEY_FILE` / `JWT_PUBLIC_KEY_FILE` env vars.
    /// Falls back to [`Self::generate()`] with a warning if unset.
    pub fn from_env() -> Result<Self> {
        let private_path = std::env::var("JWT_PRIVATE_KEY_FILE")
            .ok()
            .filter(|s| !s.is_empty());
        let public_path = std::env::var("JWT_PUBLIC_KEY_FILE")
            .ok()
            .filter(|s| !s.is_empty());

        match (private_path, public_path) {
            (Some(priv_path), Some(pub_path)) => {
                let private_pem = std::fs::read(&priv_path).map_err(|e| {
                    anyhow::anyhow!("failed to read JWT private key from {priv_path}: {e}")
                })?;
                let public_pem = std::fs::read(&pub_path).map_err(|e| {
                    anyhow::anyhow!("failed to read JWT public key from {pub_path}: {e}")
                })?;
                Self::from_pem(&private_pem, &public_pem)
            }
            (None, None) => {
                tracing::warn!(
                    "JWT_PRIVATE_KEY_FILE and JWT_PUBLIC_KEY_FILE not set; \
                     generating ephemeral RSA key pair (NOT for production)"
                );
                Self::generate()
            }
            _ => anyhow::bail!(
                "both JWT_PRIVATE_KEY_FILE and JWT_PUBLIC_KEY_FILE must be set (only one was provided)"
            ),
        }
    }

    /// Create a signed access token (15-min TTL).
    pub fn generate_access_token(&self, user_id: Uuid) -> Result<String> {
        self.generate_token(user_id, TokenType::Access, ACCESS_TOKEN_TTL_SECS)
    }

    /// Create a signed refresh token (7-day TTL).
    pub fn generate_refresh_token(&self, user_id: Uuid) -> Result<String> {
        self.generate_token(user_id, TokenType::Refresh, REFRESH_TOKEN_TTL_SECS)
    }

    /// Validate a JWT: verify RS256 signature and expiration.
    /// Returns decoded [`Claims`]. Does not enforce `token_type`.
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let validation = Validation::new(Algorithm::RS256);
        let token_data = decode::<Claims>(token, &self.decoding, &validation)
            .map_err(|e| anyhow::anyhow!("token validation failed: {e}"))?;
        Ok(token_data.claims)
    }

    fn generate_token(
        &self,
        user_id: Uuid,
        token_type: TokenType,
        ttl_secs: u64,
    ) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow::anyhow!("system clock error: {e}"))?;
        let iat = now.as_secs() as usize;
        let exp = (now.as_secs() + ttl_secs) as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            exp,
            iat,
            jti: Uuid::new_v4().to_string(),
            token_type,
        };

        let header = Header::new(Algorithm::RS256);
        encode(&header, &claims, &self.encoding)
            .map_err(|e| anyhow::anyhow!("failed to encode JWT: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_keys() -> JwtKeys {
        JwtKeys::generate().unwrap()
    }

    #[test]
    fn generate_access_token_produces_valid_jwt() {
        let keys = test_keys();
        let token = keys.generate_access_token(Uuid::new_v4()).unwrap();
        assert_eq!(token.matches('.').count(), 2);
    }

    #[test]
    fn generate_refresh_token_produces_valid_jwt() {
        let keys = test_keys();
        let token = keys.generate_refresh_token(Uuid::new_v4()).unwrap();
        assert_eq!(token.matches('.').count(), 2);
    }

    #[test]
    fn valid_access_token_validates_with_correct_claims() {
        let keys = test_keys();
        let user_id = Uuid::new_v4();
        let token = keys.generate_access_token(user_id).unwrap();

        let claims = keys.validate_token(&token).unwrap();
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.token_type, TokenType::Access);
        assert!(!claims.jti.is_empty());
        assert!(claims.iat <= claims.exp);
        assert_eq!(claims.exp - claims.iat, ACCESS_TOKEN_TTL_SECS as usize);
    }

    #[test]
    fn valid_refresh_token_validates_with_correct_claims() {
        let keys = test_keys();
        let user_id = Uuid::new_v4();
        let token = keys.generate_refresh_token(user_id).unwrap();

        let claims = keys.validate_token(&token).unwrap();
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.token_type, TokenType::Refresh);
        assert_eq!(claims.exp - claims.iat, REFRESH_TOKEN_TTL_SECS as usize);
    }

    #[test]
    fn expired_token_rejected() {
        let keys = test_keys();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let claims = Claims {
            sub: Uuid::new_v4().to_string(),
            exp: (now - 120) as usize,
            iat: (now - 180) as usize,
            jti: Uuid::new_v4().to_string(),
            token_type: TokenType::Access,
        };
        let token = encode(&Header::new(Algorithm::RS256), &claims, &keys.encoding).unwrap();

        assert!(keys.validate_token(&token).is_err());
    }

    #[test]
    fn wrong_key_pair_rejected() {
        let keys_a = test_keys();
        let keys_b = test_keys();

        let token = keys_a.generate_access_token(Uuid::new_v4()).unwrap();
        assert!(keys_b.validate_token(&token).is_err());
    }

    #[test]
    fn jti_is_unique_across_tokens() {
        let keys = test_keys();
        let user_id = Uuid::new_v4();

        let t1 = keys.generate_access_token(user_id).unwrap();
        let t2 = keys.generate_access_token(user_id).unwrap();

        let c1 = keys.validate_token(&t1).unwrap();
        let c2 = keys.validate_token(&t2).unwrap();
        assert_ne!(c1.jti, c2.jti);
    }

    #[test]
    fn jti_is_valid_uuid_v4() {
        let keys = test_keys();
        let token = keys.generate_access_token(Uuid::new_v4()).unwrap();
        let claims = keys.validate_token(&token).unwrap();

        let jti_uuid = Uuid::parse_str(&claims.jti).unwrap();
        assert_eq!(jti_uuid.get_version_num(), 4);
    }

    #[test]
    fn from_pem_rejects_invalid_key() {
        assert!(JwtKeys::from_pem(b"not a key", b"also not a key").is_err());
    }

    #[test]
    fn access_and_refresh_tokens_have_different_types() {
        let keys = test_keys();
        let user_id = Uuid::new_v4();

        let access = keys.generate_access_token(user_id).unwrap();
        let refresh = keys.generate_refresh_token(user_id).unwrap();

        let ac = keys.validate_token(&access).unwrap();
        let rc = keys.validate_token(&refresh).unwrap();
        assert_eq!(ac.token_type, TokenType::Access);
        assert_eq!(rc.token_type, TokenType::Refresh);
    }

    #[test]
    fn tampered_payload_rejected() {
        let keys = test_keys();
        let token = keys.generate_access_token(Uuid::new_v4()).unwrap();

        // Flip a character in the payload (second segment)
        let parts: Vec<&str> = token.splitn(3, '.').collect();
        let mut payload_bytes = parts[1].as_bytes().to_vec();
        payload_bytes[0] ^= 0xFF;
        let tampered = format!(
            "{}.{}.{}",
            parts[0],
            String::from_utf8_lossy(&payload_bytes),
            parts[2]
        );

        assert!(keys.validate_token(&tampered).is_err());
    }

    #[test]
    fn validate_empty_string_rejected() {
        let keys = test_keys();
        assert!(keys.validate_token("").is_err());
    }

    #[test]
    fn validate_garbage_string_rejected() {
        let keys = test_keys();
        assert!(keys.validate_token("abc.def.ghi").is_err());
    }

    #[test]
    fn sub_contains_exact_user_id() {
        let keys = test_keys();
        let user_id = Uuid::new_v4();

        let access = keys.generate_access_token(user_id).unwrap();
        let refresh = keys.generate_refresh_token(user_id).unwrap();

        let ac = keys.validate_token(&access).unwrap();
        let rc = keys.validate_token(&refresh).unwrap();

        // sub is a parseable UUID matching the input
        assert_eq!(Uuid::parse_str(&ac.sub).unwrap(), user_id);
        assert_eq!(Uuid::parse_str(&rc.sub).unwrap(), user_id);
    }
}
