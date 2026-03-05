use anyhow::Result;
use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::{self, SaltString, rand_core::OsRng},
};

/// OWASP 2026 recommended Argon2id parameters
/// m=19456 (19 MiB), t=2 iterations, p=1 parallelism
fn argon2_instance() -> Result<Argon2<'static>> {
    let params = Params::new(19456, 2, 1, None)
        .map_err(|e| anyhow::anyhow!("invalid Argon2id params: {e}"))?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = argon2_instance()?
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("failed to hash password: {e}"))?;
    Ok(hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| anyhow::anyhow!("failed to parse password hash: {e}"))?;
    match argon2_instance()?.verify_password(password.as_bytes(), &parsed) {
        Ok(()) => Ok(true),
        Err(password_hash::Error::Password) => Ok(false),
        Err(e) => Err(anyhow::anyhow!("password verification error: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_produces_valid_argon2id_string() {
        let hash = hash_password("test-password").unwrap();
        assert!(hash.starts_with("$argon2id$"));
        // Verify the hash is parseable
        PasswordHash::new(&hash).unwrap();
    }

    #[test]
    fn verify_correct_password() {
        let hash = hash_password("correct-password").unwrap();
        assert!(verify_password("correct-password", &hash).unwrap());
    }

    #[test]
    fn verify_wrong_password() {
        let hash = hash_password("correct-password").unwrap();
        assert!(!verify_password("wrong-password", &hash).unwrap());
    }

    #[test]
    fn hash_produces_unique_salts() {
        let hash1 = hash_password("same-password").unwrap();
        let hash2 = hash_password("same-password").unwrap();
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn empty_password_hashes_and_verifies() {
        let hash = hash_password("").unwrap();
        assert!(verify_password("", &hash).unwrap());
        assert!(!verify_password("not-empty", &hash).unwrap());
    }

    #[test]
    fn unicode_password_hashes_and_verifies() {
        let password = "m0t-dé-pàsse-🔐-密码";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong", &hash).unwrap());
    }

    #[test]
    fn long_password_hashes_and_verifies() {
        let password = "a".repeat(1_000_000);
        let hash = hash_password(&password).unwrap();
        assert!(verify_password(&password, &hash).unwrap());
        assert!(!verify_password("short", &hash).unwrap());
    }

    #[test]
    fn verify_rejects_invalid_hash_string() {
        let result = verify_password("password", "not-a-valid-hash");
        assert!(result.is_err());
    }

    #[test]
    fn verify_errors_on_wrong_algorithm_hash() {
        // Argon2d hash (not Argon2id) — should error, not silently return false
        let argon2d_hash = "$argon2d$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$dGVzdGhhc2g";
        let result = verify_password("password", argon2d_hash);
        assert!(result.is_err());
    }
}
