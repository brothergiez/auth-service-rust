//! HTTP middleware — one module per concern (`auth`, `request_log`, …).

mod redact;

pub mod auth;
pub mod request_log;

pub use auth::{require_jwt, AuthUser};
pub use request_log::log_request;
