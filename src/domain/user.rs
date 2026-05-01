use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Domain entity — independent of persistence and transport layers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: ObjectId,
    pub email: String,
    pub password_hash: String,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
}
