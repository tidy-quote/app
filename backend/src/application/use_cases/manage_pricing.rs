use chrono::Utc;

use crate::application::ports::{PricingStore, StoreError};
use crate::domain::entities::*;
use crate::domain::value_objects::*;

#[derive(Debug, thiserror::Error)]
pub enum ManagePricingError {
    #[error("store error: {0}")]
    Store(#[from] StoreError),
}

pub struct ManagePricingUseCase<'a> {
    store: &'a dyn PricingStore,
}

impl<'a> ManagePricingUseCase<'a> {
    pub fn new(store: &'a dyn PricingStore) -> Self {
        Self { store }
    }

    pub async fn get_template(
        &self,
        user_id: &UserId,
    ) -> Result<Option<PricingTemplate>, ManagePricingError> {
        Ok(self.store.get_template(user_id).await?)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn save_template(
        &self,
        user_id: UserId,
        currency: String,
        country: String,
        minimum_callout: f64,
        categories: Vec<ServiceCategory>,
        add_ons: Vec<AddOn>,
        custom_notes: String,
    ) -> Result<PricingTemplate, ManagePricingError> {
        let existing = self.store.get_template(&user_id).await?;
        let now = Utc::now();

        let template = match existing {
            Some(mut t) => {
                t.currency = currency;
                t.country = country;
                t.minimum_callout = minimum_callout;
                t.categories = categories;
                t.add_ons = add_ons;
                t.custom_notes = custom_notes;
                t.updated_at = now;
                t
            }
            None => PricingTemplate {
                id: TemplateId::generate(),
                user_id,
                currency,
                country,
                minimum_callout,
                categories,
                add_ons,
                custom_notes,
                created_at: now,
                updated_at: now,
            },
        };

        self.store.save_template(&template).await?;
        Ok(template)
    }
}
