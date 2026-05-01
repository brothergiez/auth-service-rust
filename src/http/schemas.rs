use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

use crate::domain::User;

fn validate_password(password: &str) -> Result<(), ValidationError> {
    if password.len() < 8 {
        return Err(ValidationError::new("password_too_short"));
    }
    Ok(())
}

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[validate(email(message = "invalid email"))]
    pub email: String,
    #[validate(custom(function = "validate_password"))]
    pub password: String,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(email(message = "invalid email"))]
    pub email: String,
    #[validate(length(min = 1, message = "password is required"))]
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserPublic {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
}

impl From<&User> for UserPublic {
    fn from(u: &User) -> Self {
        UserPublic {
            id: u.id.to_hex(),
            email: u.email.clone(),
            name: u.name.clone(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: String,
    #[schema(example = 3600)]
    pub expires_in: u64,
    pub user: UserPublic,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
}
