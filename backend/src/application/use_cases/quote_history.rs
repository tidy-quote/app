use crate::application::ports::{QuoteStore, StoreError};
use crate::domain::entities::QuoteDraft;
use crate::domain::value_objects::{QuoteId, UserId};

#[derive(Debug, thiserror::Error)]
pub enum QuoteHistoryError {
    #[error("store error: {0}")]
    Store(#[from] StoreError),
}

pub struct QuoteHistoryUseCase<'a> {
    store: &'a dyn QuoteStore,
}

impl<'a> QuoteHistoryUseCase<'a> {
    pub fn new(store: &'a dyn QuoteStore) -> Self {
        Self { store }
    }

    pub async fn list_quotes(
        &self,
        user_id: &UserId,
        page: u32,
        limit: u32,
    ) -> Result<Vec<QuoteDraft>, QuoteHistoryError> {
        Ok(self.store.list_quotes(user_id, page, limit).await?)
    }

    pub async fn get_quote(
        &self,
        quote_id: &QuoteId,
        user_id: &UserId,
    ) -> Result<Option<QuoteDraft>, QuoteHistoryError> {
        Ok(self.store.get_quote(quote_id, user_id).await?)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use async_trait::async_trait;
    use chrono::Utc;

    use super::*;
    use crate::domain::entities::*;
    use crate::domain::value_objects::*;

    struct MockQuoteStore {
        quotes: Mutex<Vec<QuoteDraft>>,
    }

    impl MockQuoteStore {
        fn new(quotes: Vec<QuoteDraft>) -> Self {
            Self {
                quotes: Mutex::new(quotes),
            }
        }
    }

    #[async_trait]
    impl QuoteStore for MockQuoteStore {
        async fn save_quote(&self, quote: &QuoteDraft) -> Result<(), StoreError> {
            self.quotes.lock().unwrap().push(quote.clone());
            Ok(())
        }

        async fn list_quotes(
            &self,
            user_id: &UserId,
            page: u32,
            limit: u32,
        ) -> Result<Vec<QuoteDraft>, StoreError> {
            let quotes = self.quotes.lock().unwrap();
            let owned: Vec<_> = quotes
                .iter()
                .filter(|q| q.user_id == *user_id)
                .cloned()
                .collect();

            let skip = (page.saturating_sub(1) as usize) * (limit as usize);
            let result = owned.into_iter().skip(skip).take(limit as usize).collect();
            Ok(result)
        }

        async fn get_quote(
            &self,
            quote_id: &QuoteId,
            user_id: &UserId,
        ) -> Result<Option<QuoteDraft>, StoreError> {
            let quotes = self.quotes.lock().unwrap();
            Ok(quotes
                .iter()
                .find(|q| q.id == *quote_id && q.user_id == *user_id)
                .cloned())
        }
    }

    fn make_quote(user_id: &str) -> QuoteDraft {
        QuoteDraft {
            id: QuoteId::generate(),
            lead_id: LeadId::generate(),
            user_id: UserId::new(user_id),
            job_summary: JobSummary {
                service_type: "Deep Clean".to_string(),
                property_size: None,
                requested_date: None,
                requested_time: None,
                missing_info: Vec::new(),
                extracted_details: HashMap::new(),
            },
            estimated_price: 100.0,
            price_breakdown: Vec::new(),
            assumptions: Vec::new(),
            follow_up_message: "Thanks!".to_string(),
            clarification_message: None,
            tone: ToneOption::Friendly,
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn lists_quotes_for_user() {
        let quotes = vec![
            make_quote("user-1"),
            make_quote("user-1"),
            make_quote("user-2"),
        ];
        let store = MockQuoteStore::new(quotes);
        let use_case = QuoteHistoryUseCase::new(&store);

        let result = use_case
            .list_quotes(&UserId::new("user-1"), 1, 20)
            .await
            .unwrap();

        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn paginates_results() {
        let quotes: Vec<_> = (0..5).map(|_| make_quote("user-1")).collect();
        let store = MockQuoteStore::new(quotes);
        let use_case = QuoteHistoryUseCase::new(&store);

        let page1 = use_case
            .list_quotes(&UserId::new("user-1"), 1, 2)
            .await
            .unwrap();
        let page2 = use_case
            .list_quotes(&UserId::new("user-1"), 2, 2)
            .await
            .unwrap();
        let page3 = use_case
            .list_quotes(&UserId::new("user-1"), 3, 2)
            .await
            .unwrap();

        assert_eq!(page1.len(), 2);
        assert_eq!(page2.len(), 2);
        assert_eq!(page3.len(), 1);
    }

    #[tokio::test]
    async fn returns_empty_for_unknown_user() {
        let quotes = vec![make_quote("user-1")];
        let store = MockQuoteStore::new(quotes);
        let use_case = QuoteHistoryUseCase::new(&store);

        let result = use_case
            .list_quotes(&UserId::new("unknown"), 1, 20)
            .await
            .unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn gets_quote_by_id_for_owner() {
        let quote = make_quote("user-1");
        let quote_id = quote.id.clone();
        let store = MockQuoteStore::new(vec![quote]);
        let use_case = QuoteHistoryUseCase::new(&store);

        let result = use_case
            .get_quote(&quote_id, &UserId::new("user-1"))
            .await
            .unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().id, quote_id);
    }

    #[tokio::test]
    async fn returns_none_for_wrong_owner() {
        let quote = make_quote("user-1");
        let quote_id = quote.id.clone();
        let store = MockQuoteStore::new(vec![quote]);
        let use_case = QuoteHistoryUseCase::new(&store);

        let result = use_case
            .get_quote(&quote_id, &UserId::new("user-2"))
            .await
            .unwrap();

        assert!(result.is_none());
    }
}
