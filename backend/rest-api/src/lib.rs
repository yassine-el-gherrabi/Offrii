pub mod config;
pub mod dto;
pub mod errors;
pub mod handlers;
pub mod jobs;
pub mod middleware;
pub mod models;
pub mod openapi;
pub mod repositories;
pub mod services;
pub mod traits;
pub mod utils;

use std::sync::Arc;

use crate::utils::jwt::JwtKeys;

#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<dyn traits::AuthService>,
    pub jwt: Arc<JwtKeys>,
    pub db: sqlx::PgPool,
    pub redis: redis::Client,
    pub health: Arc<dyn traits::HealthCheck>,
    pub items: Arc<dyn traits::ItemService>,
    pub categories: Arc<dyn traits::CategoryService>,
    pub users: Arc<dyn traits::UserService>,
    pub push_tokens: Arc<dyn traits::PushTokenService>,
    pub share_links: Arc<dyn traits::ShareLinkService>,
    pub circles: Arc<dyn traits::CircleService>,
    pub friends: Arc<dyn traits::FriendService>,
    pub community_wishes: Arc<dyn traits::CommunityWishService>,
    pub wish_messages: Arc<dyn traits::WishMessageService>,
    pub uploads: Arc<dyn traits::UploadService>,
    pub notifications: Arc<dyn traits::NotificationRepo>,
    pub share_rules: Arc<dyn traits::CircleShareRuleRepo>,
    pub circle_events: Arc<dyn traits::CircleEventRepo>,
    pub app_base_url: String,
}
