use async_trait::async_trait;
use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use sqlx::MySqlPool;

use crate::domain::User;
use crate::error::AppError;
use crate::repository::UserRepository;

#[derive(sqlx::FromRow)]
struct UserRow {
    id: String,
    email: String,
    password_hash: String,
    name: Option<String>,
    created_at: DateTime<Utc>,
}

fn row_to_user(r: UserRow) -> Result<User, AppError> {
    Ok(User {
        id: ObjectId::parse_str(&r.id)
            .map_err(|_| AppError::Internal("invalid user id stored in MySQL".into()))?,
        email: r.email,
        password_hash: r.password_hash,
        name: r.name,
        created_at: r.created_at,
    })
}

pub struct MysqlUserRepository {
    pool: MySqlPool,
}

impl MysqlUserRepository {
    /// Connect and apply embedded SQL migrations (`migrations/`).
    pub async fn connect_and_migrate(database_url: &str) -> Result<Self, AppError> {
        let pool = MySqlPool::connect(database_url)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| AppError::Database(format!("migration failed: {e}")))?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl UserRepository for MysqlUserRepository {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, email, password_hash, name, created_at FROM users WHERE email = ?",
        )
        .bind(email.to_lowercase())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_user).transpose()
    }

    async fn find_by_id(&self, id: ObjectId) -> Result<Option<User>, AppError> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, email, password_hash, name, created_at FROM users WHERE id = ?",
        )
        .bind(id.to_hex())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_user).transpose()
    }

    async fn create(&self, user: &User) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, name, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(user.id.to_hex())
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.name)
        .bind(user.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
