use async_trait::async_trait;
use mongodb::bson::{self, doc};
use mongodb::{Client, Collection, Database};

use mongodb::options::FindOptions;

use crate::application::ports::{PricingStore, QuoteStore, StoreError, TokenStore, UserStore};
use crate::domain::entities::{
    PricingTemplate, QuoteDraft, SubscriptionStatus, TokenPurpose, User, VerificationToken,
};
use crate::domain::value_objects::{QuoteId, UserId};

const DEFAULT_DB_NAME: &str = "tidy-quote";
const COLLECTION_PRICING_TEMPLATES: &str = "pricing_templates";
const COLLECTION_USERS: &str = "users";
const COLLECTION_QUOTES: &str = "quotes";
const COLLECTION_VERIFICATION_TOKENS: &str = "verification_tokens";

pub struct MongoStore {
    pricing_collection: Collection<PricingTemplate>,
    users_collection: Collection<User>,
    quotes_collection: Collection<QuoteDraft>,
    tokens_collection: Collection<VerificationToken>,
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
        let quotes_collection = db.collection::<QuoteDraft>(COLLECTION_QUOTES);
        let tokens_collection =
            db.collection::<VerificationToken>(COLLECTION_VERIFICATION_TOKENS);

        Self::ensure_user_indexes(&users_collection).await?;
        Self::ensure_quote_indexes(&quotes_collection).await?;

        Ok(Self {
            pricing_collection,
            users_collection,
            quotes_collection,
            tokens_collection,
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

    async fn ensure_quote_indexes(
        collection: &Collection<QuoteDraft>,
    ) -> Result<(), StoreError> {
        use mongodb::IndexModel;

        let index = IndexModel::builder()
            .keys(doc! { "userId": 1, "createdAt": -1 })
            .build();

        collection
            .create_index(index)
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl QuoteStore for MongoStore {
    async fn save_quote(&self, quote: &QuoteDraft) -> Result<(), StoreError> {
        self.quotes_collection
            .insert_one(quote)
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn list_quotes(
        &self,
        user_id: &UserId,
        page: u32,
        limit: u32,
    ) -> Result<Vec<QuoteDraft>, StoreError> {
        use futures::TryStreamExt;

        let filter = doc! { "userId": user_id.as_str() };
        let skip = (page.saturating_sub(1)) as u64 * limit as u64;

        let options = FindOptions::builder()
            .sort(doc! { "createdAt": -1 })
            .skip(skip)
            .limit(limit as i64)
            .build();

        let cursor = self
            .quotes_collection
            .find(filter)
            .with_options(options)
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))?;

        let quotes: Vec<QuoteDraft> = cursor
            .try_collect()
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))?;

        Ok(quotes)
    }

    async fn get_quote(
        &self,
        quote_id: &QuoteId,
        user_id: &UserId,
    ) -> Result<Option<QuoteDraft>, StoreError> {
        let filter = doc! { "id": quote_id.as_str(), "userId": user_id.as_str() };

        self.quotes_collection
            .find_one(filter)
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))
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

    async fn set_email_verified(&self, user_id: &UserId) -> Result<(), StoreError> {
        let filter = doc! { "id": user_id.as_str() };
        let update = doc! { "$set": { "email_verified": true } };

        self.users_collection
            .update_one(filter, update)
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn update_password(
        &self,
        user_id: &UserId,
        password_hash: &str,
    ) -> Result<(), StoreError> {
        let filter = doc! { "id": user_id.as_str() };
        let update = doc! { "$set": { "password_hash": password_hash } };

        self.users_collection
            .update_one(filter, update)
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, user_id: &UserId) -> Result<Option<User>, StoreError> {
        let filter = doc! { "id": user_id.as_str() };

        self.users_collection
            .find_one(filter)
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))
    }

    async fn update_subscription(
        &self,
        user_id: &UserId,
        stripe_customer_id: &str,
        status: SubscriptionStatus,
        plan: Option<String>,
    ) -> Result<(), StoreError> {
        let filter = doc! { "id": user_id.as_str() };
        let status_bson =
            bson::to_bson(&status).map_err(|e| StoreError::Serialization(e.to_string()))?;
        let plan_bson =
            bson::to_bson(&plan).map_err(|e| StoreError::Serialization(e.to_string()))?;

        let update = doc! {
            "$set": {
                "stripe_customer_id": stripe_customer_id,
                "subscription_status": status_bson,
                "subscription_plan": plan_bson,
            }
        };

        self.users_collection
            .update_one(filter, update)
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn find_by_stripe_customer_id(
        &self,
        customer_id: &str,
    ) -> Result<Option<User>, StoreError> {
        let filter = doc! { "stripe_customer_id": customer_id };

        self.users_collection
            .find_one(filter)
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))
    }
}

#[async_trait]
impl TokenStore for MongoStore {
    async fn store_token(&self, token: &VerificationToken) -> Result<(), StoreError> {
        self.tokens_collection
            .insert_one(token)
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn find_valid_token(
        &self,
        token_hash: &str,
        purpose: TokenPurpose,
    ) -> Result<Option<VerificationToken>, StoreError> {
        let purpose_bson =
            bson::to_bson(&purpose).map_err(|e| StoreError::Serialization(e.to_string()))?;

        let filter = doc! {
            "token_hash": token_hash,
            "purpose": purpose_bson,
            "used": false,
            "expires_at": { "$gt": bson::DateTime::now() },
        };

        self.tokens_collection
            .find_one(filter)
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))
    }

    async fn mark_token_used(&self, token_hash: &str) -> Result<(), StoreError> {
        let filter = doc! { "token_hash": token_hash };
        let update = doc! { "$set": { "used": true } };

        self.tokens_collection
            .update_one(filter, update)
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))?;

        Ok(())
    }
}
