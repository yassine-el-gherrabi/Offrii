pub mod config;
pub mod errors;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod repositories;
pub mod services;
pub mod utils;

use std::sync::Arc;

use sqlx::PgPool;

use crate::utils::jwt::JwtKeys;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: redis::Client,
    pub jwt: Arc<JwtKeys>,
}
