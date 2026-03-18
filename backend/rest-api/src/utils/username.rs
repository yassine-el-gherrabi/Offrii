/// Usernames that cannot be registered — prevents impersonation, URL conflicts, and confusion.
const RESERVED_USERNAMES: &[&str] = &[
    // App identity
    "offrii",
    "admin",
    "administrator",
    "support",
    "help",
    "team",
    "staff",
    "moderator",
    "official",
    // System terms
    "system",
    "bot",
    "notification",
    "notifications",
    "noreply",
    "mailer",
    "root",
    "superuser",
    // URL/route conflicts
    "api",
    "app",
    "www",
    "web",
    "mail",
    "cdn",
    "static",
    "auth",
    "login",
    "register",
    "signup",
    "signin",
    "logout",
    "settings",
    "profile",
    "account",
    "dashboard",
    "home",
    "search",
    "explore",
    "discover",
    "feed",
    "cercles",
    "circles",
    "envies",
    "wishes",
    "wishlist",
    "entraide",
    "proches",
    "share",
    "shared",
    "join",
    "invite",
    // Misleading generics
    "everyone",
    "anonymous",
    "unknown",
    "deleted",
    "null",
    "undefined",
    "test",
    "demo",
    "example",
];

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

/// Check if a username is reserved (case-insensitive).
pub fn is_reserved_username(s: &str) -> bool {
    let lower = s.to_lowercase();
    RESERVED_USERNAMES.contains(&lower.as_str())
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

    #[test]
    fn reserved_usernames_blocked() {
        assert!(is_reserved_username("admin"));
        assert!(is_reserved_username("offrii"));
        assert!(is_reserved_username("support"));
        assert!(is_reserved_username("system"));
        assert!(is_reserved_username("api"));
        assert!(is_reserved_username("test"));
        assert!(is_reserved_username("null"));
    }

    #[test]
    fn normal_usernames_not_reserved() {
        assert!(!is_reserved_username("alice"));
        assert!(!is_reserved_username("bob_123"));
        assert!(!is_reserved_username("yassine"));
        assert!(!is_reserved_username("marie_d"));
    }
}
