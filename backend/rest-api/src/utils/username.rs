/// Validate username format: ^[a-z][a-z0-9_]{2,29}$
pub fn is_valid_username(s: &str) -> bool {
    if s.len() < 3 || s.len() > 30 {
        return false;
    }
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_usernames() {
        assert!(is_valid_username("abc"));
        assert!(is_valid_username("alice_e2e"));
        assert!(is_valid_username("user123"));
        assert!(is_valid_username("a_b_c"));
        assert!(is_valid_username(&format!("a{}", "b".repeat(29))));
    }

    #[test]
    fn invalid_usernames() {
        assert!(!is_valid_username("ab")); // too short
        assert!(!is_valid_username(&"a".repeat(31))); // too long
        assert!(!is_valid_username("1abc")); // starts with digit
        assert!(!is_valid_username("_abc")); // starts with underscore
        assert!(!is_valid_username("Abc")); // uppercase
        assert!(!is_valid_username("abc!")); // special char
        assert!(!is_valid_username("")); // empty
    }
}
