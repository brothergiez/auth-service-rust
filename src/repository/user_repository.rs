use async_trait::async_trait;
use bson::oid::ObjectId;

use crate::domain::User;
use crate::error::AppError;

/// Persistence abstraction — handlers/services depend on this trait (Dependency Inversion).
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
    async fn find_by_id(&self, id: ObjectId) -> Result<Option<User>, AppError>;
    async fn create(&self, user: &User) -> Result<(), AppError>;
}
