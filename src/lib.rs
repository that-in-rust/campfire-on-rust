pub mod models;
pub mod services;
pub mod errors;
pub mod handlers;
pub mod database;
pub mod middleware;

pub use database::Database;
pub use services::auth::{AuthService, AuthServiceTrait};

use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub auth_service: Arc<dyn AuthServiceTrait>,
}