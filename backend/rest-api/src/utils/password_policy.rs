use super::{common_passwords, hibp};

#[derive(Debug, PartialEq, Eq)]
pub enum PasswordPolicyViolation {
    TooShort,
    TooLong,
    Common,
    Breached,
}

/// Validate a password against OWASP 2024 recommendations.
///
/// Checks, in order:
/// 1. Length (8–128)
/// 2. Common passwords (top 10k)
/// 3. HIBP breached-passwords database
pub async fn check(password: &str) -> Result<(), PasswordPolicyViolation> {
    let char_count = password.chars().count();
    if char_count < 8 {
        return Err(PasswordPolicyViolation::TooShort);
    }
    if char_count > 128 {
        return Err(PasswordPolicyViolation::TooLong);
    }

    if common_passwords::is_common(password) {
        return Err(PasswordPolicyViolation::Common);
    }

    if hibp::is_breached(password).await.unwrap_or(false) {
        return Err(PasswordPolicyViolation::Breached);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rejects_too_short() {
        assert_eq!(check("short").await, Err(PasswordPolicyViolation::TooShort));
    }

    #[tokio::test]
    async fn rejects_too_long() {
        let long = "a".repeat(129);
        assert_eq!(check(&long).await, Err(PasswordPolicyViolation::TooLong));
    }

    #[tokio::test]
    async fn rejects_common_password() {
        assert_eq!(
            check("password").await,
            Err(PasswordPolicyViolation::Common)
        );
        assert_eq!(
            check("12345678").await,
            Err(PasswordPolicyViolation::Common)
        );
    }

    #[tokio::test]
    async fn accepts_strong_unique_password() {
        // This random string is not common and unlikely to be breached
        // HIBP call may fail-open in CI which is fine
        assert_eq!(check("xK9mQ2vL7nB4pR8sW3").await, Ok(()));
    }

    #[tokio::test]
    async fn multibyte_password_counts_chars_not_bytes() {
        // "héllo🌍wd" = 9 chars but 14 bytes — should pass the >=8 char check
        assert_ne!(
            check("héllo🌍wd").await,
            Err(PasswordPolicyViolation::TooShort)
        );
    }

    #[tokio::test]
    async fn multibyte_short_password_rejected() {
        // "à🌍🎉" = 3 chars — should be rejected as too short
        assert_eq!(check("à🌍🎉").await, Err(PasswordPolicyViolation::TooShort));
    }

    #[tokio::test]
    async fn boundary_exactly_8_chars_accepted() {
        assert_ne!(
            check("abcdefgh").await,
            Err(PasswordPolicyViolation::TooShort)
        );
    }

    #[tokio::test]
    async fn boundary_exactly_128_chars_accepted() {
        let pass = "a".repeat(128);
        assert_ne!(check(&pass).await, Err(PasswordPolicyViolation::TooLong));
    }
}
