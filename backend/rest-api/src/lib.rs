pub mod config;
pub mod dto;
pub mod errors;
pub mod handlers;
pub mod jobs;
pub mod middleware;
pub mod models;
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
    pub redis: redis::Client,
    pub health: Arc<dyn traits::HealthCheck>,
    pub items: Arc<dyn traits::ItemService>,
    pub categories: Arc<dyn traits::CategoryService>,
    pub users: Arc<dyn traits::UserService>,
    pub push_tokens: Arc<dyn traits::PushTokenService>,
    pub share_links: Arc<dyn traits::ShareLinkService>,
    pub circles: Arc<dyn traits::CircleService>,
}
