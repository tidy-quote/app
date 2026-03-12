use async_trait::async_trait;
use mongodb::bson::doc;
use mongodb::{Client, Collection, Database};

use crate::application::ports::{PricingStore, StoreError, UserStore};
use crate::domain::entities::{PricingTemplate, User};
use crate::domain::value_objects::UserId;

const DEFAULT_DB_NAME: &str = "tidyquote";
const COLLECTION_PRICING_TEMPLATES: &str = "pricing_templates";
const COLLECTION_USERS: &str = "users";

pub struct MongoStore {
    pricing_collection: Collection<PricingTemplate>,
    users_collection: Collection<User>,
}

impl MongoStore {
    pub async fn new(connection_uri: &str) -> Result<Self, StoreError> {
        Self::with_database(connection_uri, DEFAULT_DB_NAME).await
    }

    pub async fn with_database(connection_uri: &str, db_name: &str) -> Result<Self, StoreError> {
        let client = Client::with_uri_str(connection_uri)
            .await
            .map_err(|e| StoreError::Connection(e.to_string()))?;

        let db = client.database(db_name);
        Self::from_database(&db).await
    }

    async fn from_database(db: &Database) -> Result<Self, StoreError> {
        let pricing_collection = db.collection::<PricingTemplate>(COLLECTION_PRICING_TEMPLATES);
        let users_collection = db.collection::<User>(COLLECTION_USERS);

        Self::ensure_user_indexes(&users_collection).await?;

        Ok(Self {
            pricing_collection,
            users_collection,
        })
    }

    async fn ensure_user_indexes(collection: &Collection<User>) -> Result<(), StoreError> {
        use mongodb::IndexModel;

        let index = IndexModel::builder()
            .keys(doc! { "email": 1 })
            .options(
                mongodb::options::IndexOptions::builder()
                    .unique(true)
                    .build(),
            )
            .build();

        collection
            .create_index(index)
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl PricingStore for MongoStore {
    async fn get_template(&self, user_id: &UserId) -> Result<Option<PricingTemplate>, StoreError> {
        let filter = doc! { "userId": user_id.as_str() };

        self.pricing_collection
            .find_one(filter)
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))
    }

    async fn save_template(&self, template: &PricingTemplate) -> Result<(), StoreError> {
        let filter = doc! { "userId": template.user_id.as_str() };

        self.pricing_collection
            .replace_one(filter, template)
            .upsert(true)
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl UserStore for MongoStore {
    async fn create_user(&self, user: &User) -> Result<(), StoreError> {
        self.users_collection.insert_one(user).await.map_err(|e| {
            if e.to_string().contains("E11000") {
                StoreError::DuplicateEmail(user.email.clone())
            } else {
                StoreError::Internal(e.to_string())
            }
        })?;

        Ok(())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, StoreError> {
        let filter = doc! { "email": email };

        self.users_collection
            .find_one(filter)
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))
    }
}
