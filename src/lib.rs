pub mod models;
pub mod services;
pub mod errors;
pub mod handlers;
pub mod database;
pub mod middleware;
pub mod rich_text;
pub mod sounds;

pub use database::CampfireDatabase;
pub use services::auth::{AuthService, AuthServiceTrait};
pub use services::room::{RoomService, RoomServiceTrait};
pub use services::message::{MessageService, MessageServiceTrait};
pub use services::connection::{ConnectionManager, ConnectionManagerImpl};
pub use services::search::{SearchService, SearchServiceTrait};

use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: CampfireDatabase,
    pub auth_service: Arc<dyn AuthServiceTrait>,
    pub room_service: Arc<dyn RoomServiceTrait>,
    pub message_service: Arc<dyn MessageServiceTrait>,
    pub search_service: Arc<dyn services::search::SearchServiceTrait>,
}