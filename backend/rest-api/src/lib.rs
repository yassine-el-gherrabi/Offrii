pub mod config;
pub mod errors;
pub mod handlers;
pub mod utils;

use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: redis::Client,
}
