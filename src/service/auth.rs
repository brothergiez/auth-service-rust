use std::sync::Arc;

use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use async_trait::async_trait;
use bson::oid::ObjectId;
use chrono::Utc;
use rand::rngs::OsRng;
use validator::Validate;

use crate::config::AppConfig;
use crate::domain::User;
use crate::error::AppError;
use crate::http::schemas::{AuthResponse, LoginRequest, RegisterRequest, UserPublic};
use crate::jwt;
use crate::repository::UserRepository;

/// Application use-cases for authentication — isolated from HTTP and MongoDB details.
#[async_trait]
pub trait AuthService: Send + Sync {
    async fn register(&self, req: RegisterRequest) -> Result<AuthResponse, AppError>;
    async fn login(&self, req: LoginRequest) -> Result<AuthResponse, AppError>;
    async fn get_me(&self, user_id: ObjectId) -> Result<UserPublic, AppError>;
}

pub struct AuthServiceImpl<R: UserRepository + ?Sized> {
    users: Arc<R>,
    config: AppConfig,
}

impl<R: UserRepository + ?Sized> AuthServiceImpl<R> {
    pub fn new(users: Arc<R>, config: AppConfig) -> Self {
        Self { users, config }
    }

    fn hash_password(password: &str) -> Result<String, AppError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(hash.to_string())
    }

    fn verify_password(password: &str, password_hash: &str) -> Result<(), AppError> {
        let parsed = PasswordHash::new(password_hash).map_err(|e| AppError::Internal(e.to_string()))?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .map_err(|_| AppError::Unauthorized("invalid email or password".into()))
    }

    fn issue_token(&self, user_id: ObjectId) -> Result<String, AppError> {
        jwt::encode_access_token(
            user_id,
            self.config.jwt_secret.as_bytes(),
            self.config.jwt_expiration.as_secs(),
        )
    }
}

#[async_trait]
impl<R: UserRepository + Send + Sync> AuthService for AuthServiceImpl<R> {
    async fn register(&self, req: RegisterRequest) -> Result<AuthResponse, AppError> {
        req.validate()?;
        let email = req.email.trim().to_lowercase();

        if self.users.find_by_email(&email).await?.is_some() {
            return Err(AppError::Conflict("email already registered".into()));
        }

        let password_hash = Self::hash_password(&req.password)?;
        let user = User {
            id: ObjectId::new(),
            email,
            password_hash,
            name: req.name.map(|n| n.trim().to_string()).filter(|n| !n.is_empty()),
            created_at: Utc::now(),
        };

        self.users.create(&user).await?;

        let token = self.issue_token(user.id)?;
        Ok(AuthResponse {
            access_token: token,
            token_type: "Bearer".into(),
            expires_in: self.config.jwt_expiration.as_secs(),
            user: UserPublic::from(&user),
        })
    }

    async fn login(&self, req: LoginRequest) -> Result<AuthResponse, AppError> {
        req.validate()?;
        let email = req.email.trim().to_lowercase();

        let user = self
            .users
            .find_by_email(&email)
            .await?
            .ok_or_else(|| AppError::Unauthorized("invalid email or password".into()))?;

        Self::verify_password(&req.password, &user.password_hash)?;

        let token = self.issue_token(user.id)?;
        Ok(AuthResponse {
            access_token: token,
            token_type: "Bearer".into(),
            expires_in: self.config.jwt_expiration.as_secs(),
            user: UserPublic::from(&user),
        })
    }

    async fn get_me(&self, user_id: ObjectId) -> Result<UserPublic, AppError> {
        let user = self
            .users
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::Unauthorized("user not found".into()))?;
        Ok(UserPublic::from(&user))
    }
}
