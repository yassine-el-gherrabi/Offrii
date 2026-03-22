use std::collections::HashSet;
use std::sync::LazyLock;

static RAW: &str = include_str!("../../data/common-passwords.txt");

static SET: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| RAW.lines().filter(|l| !l.is_empty()).collect());

/// Returns `true` if the password (case-insensitive) is in the top-10k common passwords list.
pub fn is_common(password: &str) -> bool {
    let lower = password.to_lowercase();
    SET.contains(lower.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_common_password() {
        assert!(is_common("password"));
        assert!(is_common("123456"));
        assert!(is_common("Password")); // case-insensitive
    }

    #[test]
    fn accepts_random_string() {
        assert!(!is_common("xK9#mQ2$vL7@nB4"));
        assert!(!is_common("CorrectHorseBatteryStaple42!"));
    }

    #[test]
    fn set_is_populated() {
        assert!(
            SET.len() > 9_000,
            "expected 10k passwords, got {}",
            SET.len()
        );
    }
}
