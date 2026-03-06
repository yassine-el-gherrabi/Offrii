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
    if password.len() < 8 {
        return Err(PasswordPolicyViolation::TooShort);
    }
    if password.len() > 128 {
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
}
