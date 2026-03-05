pub mod config;
pub mod dto;
pub mod errors;
pub mod handlers;
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
    pub health: Arc<dyn traits::HealthCheck>,
}
