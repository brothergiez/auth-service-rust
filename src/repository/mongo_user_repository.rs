use async_trait::async_trait;
use bson::doc;
use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use mongodb::options::IndexOptions;
use mongodb::{Client, Collection, IndexModel};

use crate::domain::User;
use crate::error::AppError;
use crate::repository::UserRepository;

const COLLECTION: &str = "users";

#[derive(serde::Serialize, serde::Deserialize)]
struct UserDoc {
    #[serde(rename = "_id")]
    id: ObjectId,
    email: String,
    password_hash: String,
    name: Option<String>,
    created_at: DateTime<Utc>,
}

impl From<UserDoc> for User {
    fn from(d: UserDoc) -> Self {
        User {
            id: d.id,
            email: d.email,
            password_hash: d.password_hash,
            name: d.name,
            created_at: d.created_at,
        }
    }
}

impl From<&User> for UserDoc {
    fn from(u: &User) -> Self {
        UserDoc {
            id: u.id,
            email: u.email.clone(),
            password_hash: u.password_hash.clone(),
            name: u.name.clone(),
            created_at: u.created_at,
        }
    }
}

pub struct MongoUserRepository {
    collection: Collection<UserDoc>,
}

impl MongoUserRepository {
    pub async fn new(client: &Client, database_name: &str) -> Result<Self, AppError> {
        let db = client.database(database_name);
        let collection = db.collection::<UserDoc>(COLLECTION);

        let index = IndexModel::builder()
            .keys(doc! { "email": 1 })
            .options(
                IndexOptions::builder()
                    .unique(true)
                    .name(Some("email_unique".into()))
                    .build(),
            )
            .build();

        db.collection::<UserDoc>(COLLECTION)
            .create_index(index)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(Self { collection })
    }
}

#[async_trait]
impl UserRepository for MongoUserRepository {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let filter = doc! { "email": email.to_lowercase() };
        let doc = self.collection.find_one(filter).await?;
        Ok(doc.map(User::from))
    }

    async fn find_by_id(&self, id: ObjectId) -> Result<Option<User>, AppError> {
        let filter = doc! { "_id": id };
        let doc = self.collection.find_one(filter).await?;
        Ok(doc.map(User::from))
    }

    async fn create(&self, user: &User) -> Result<(), AppError> {
        let doc = UserDoc::from(user);
        self.collection.insert_one(doc).await?;
        Ok(())
    }
}
