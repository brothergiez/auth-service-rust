use std::time::{SystemTime, UNIX_EPOCH};

use bson::oid::ObjectId;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessClaims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

pub fn encode_access_token(
    user_id: ObjectId,
    secret: &[u8],
    expiration_secs: u64,
) -> Result<String, AppError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| AppError::Internal(e.to_string()))?
        .as_secs() as usize;
    let exp = now + expiration_secs as usize;
    let claims = AccessClaims {
        sub: user_id.to_hex(),
        exp,
        iat: now,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(Into::into)
}

pub fn decode_access_token(token: &str, secret: &str) -> Result<AccessClaims, AppError> {
    let token = token.trim();
    decode::<AccessClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("invalid or expired token".into()))
    .map(|d| d.claims)
}
