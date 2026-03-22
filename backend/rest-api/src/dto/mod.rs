pub mod auth;

/// Serde helper for `Option<Option<T>>` (nullable PATCH semantics).
/// - JSON field absent → `None` (don't touch)
/// - JSON field `null` → `Some(None)` (set to NULL)
/// - JSON field `"value"` → `Some(Some("value"))` (set value)
pub mod nullable {
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
    where
        T: Deserialize<'de>,
        D: Deserializer<'de>,
    {
        // If this function is called, the field was present in JSON.
        // Deserialize the value: null → None, value → Some(value).
        Ok(Some(Option::deserialize(deserializer)?))
    }
}
pub mod categories;
pub mod circles;
pub mod community_wishes;
pub mod friends;
pub mod health;
pub mod items;
pub mod notifications;
pub mod pagination;
pub mod push_tokens;
pub mod share_links;
pub mod users;
pub mod wish_messages;
