use async_trait::async_trait;
use mongodb::bson::doc;
use mongodb::{Client, Collection};

use crate::application::ports::{PricingStore, StoreError};
use crate::domain::entities::PricingTemplate;
use crate::domain::value_objects::UserId;

const DEFAULT_DB_NAME: &str = "quotesnap";
const COLLECTION_PRICING_TEMPLATES: &str = "pricing_templates";

pub struct MongoStore {
    collection: Collection<PricingTemplate>,
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
        let collection = db.collection::<PricingTemplate>(COLLECTION_PRICING_TEMPLATES);

        Ok(Self { collection })
    }
}

#[async_trait]
impl PricingStore for MongoStore {
    async fn get_template(&self, user_id: &UserId) -> Result<Option<PricingTemplate>, StoreError> {
        let filter = doc! { "user_id": user_id.as_str() };

        self.collection
            .find_one(filter)
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))
    }

    async fn save_template(&self, template: &PricingTemplate) -> Result<(), StoreError> {
        let filter = doc! { "user_id": template.user_id.as_str() };

        self.collection
            .replace_one(filter, template)
            .upsert(true)
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))?;

        Ok(())
    }
}
