use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::models::Notification;

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct NotificationResponse {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub notif_type: String,
    pub title: String,
    pub body: String,
    pub read: bool,
    pub circle_id: Option<Uuid>,
    pub item_id: Option<Uuid>,
    pub wish_id: Option<Uuid>,
    pub actor_id: Option<Uuid>,
    pub actor_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl NotificationResponse {
    pub fn from_notification(n: Notification, actor_name: Option<String>) -> Self {
        Self {
            id: n.id,
            notif_type: n.r#type,
            title: n.title,
            body: n.body,
            read: n.read,
            circle_id: n.circle_id,
            item_id: n.item_id,
            wish_id: n.wish_id,
            actor_id: n.actor_id,
            actor_name,
            created_at: n.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct UnreadCountResponse {
    pub count: i64,
}
