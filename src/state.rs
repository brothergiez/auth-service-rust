use std::sync::Arc;

use crate::service::AuthService;

/// Shared application state — inject services here for handlers (composition root).
pub struct AppState {
    pub auth: Arc<dyn AuthService>,
    /// Same secret as used to sign access tokens (middleware validates `Authorization: Bearer`).
    pub jwt_secret: String,
}

impl AppState {
    pub fn new(auth: Arc<dyn AuthService>, jwt_secret: String) -> Self {
        Self { auth, jwt_secret }
    }
}
