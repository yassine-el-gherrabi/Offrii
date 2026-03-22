use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Typed status for community wishes. Serializes to/from lowercase strings
/// and maps to VARCHAR columns in PostgreSQL.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema,
)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum WishStatus {
    Pending,
    Flagged,
    Rejected,
    Open,
    Matched,
    Fulfilled,
    Closed,
    Review,
}

impl WishStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Flagged => "flagged",
            Self::Rejected => "rejected",
            Self::Open => "open",
            Self::Matched => "matched",
            Self::Fulfilled => "fulfilled",
            Self::Closed => "closed",
            Self::Review => "review",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "flagged" => Some(Self::Flagged),
            "rejected" => Some(Self::Rejected),
            "open" => Some(Self::Open),
            "matched" => Some(Self::Matched),
            "fulfilled" => Some(Self::Fulfilled),
            "closed" => Some(Self::Closed),
            "review" => Some(Self::Review),
            _ => None,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Fulfilled | Self::Closed | Self::Rejected)
    }
}

impl std::fmt::Display for WishStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct CommunityWish {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub category: String,
    pub status: WishStatus,
    pub is_anonymous: bool,
    pub matched_with: Option<Uuid>,
    pub matched_at: Option<DateTime<Utc>>,
    pub fulfilled_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub report_count: i32,
    pub reopen_count: i32,
    pub last_reopen_at: Option<DateTime<Utc>>,
    pub moderation_note: Option<String>,
    pub image_url: Option<String>,
    pub links: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct WishReport {
    pub id: Uuid,
    pub wish_id: Uuid,
    pub reporter_id: Uuid,
    pub reason: String,
    pub details: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct WishMessage {
    pub id: Uuid,
    pub wish_id: Uuid,
    pub sender_id: Option<Uuid>,
    pub body: String,
    pub created_at: DateTime<Utc>,
}
