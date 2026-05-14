use std::sync::Arc;

use redis::aio::ConnectionManager;

use crate::service::AuthService;

/// Shared application state — inject services here for handlers (composition root).
pub struct AppState {
    pub auth: Arc<dyn AuthService>,
    /// Same secret as used to sign access tokens (middleware validates `Authorization: Bearer`).
    pub jwt_secret: String,
    /// When [`crate::config::AppConfig::redis_url`] was set, holds a multiplexed async Redis handle.
    pub redis: Option<ConnectionManager>,
}

impl AppState {
    pub fn new(
        auth: Arc<dyn AuthService>,
        jwt_secret: String,
        redis: Option<ConnectionManager>,
    ) -> Self {
        Self {
            auth,
            jwt_secret,
            redis,
        }
    }
}
