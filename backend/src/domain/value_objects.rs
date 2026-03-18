use serde::{Deserialize, Serialize};

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(String);

        impl $name {
            pub fn new(id: impl Into<String>) -> Self {
                Self(id.into())
            }

            pub fn generate() -> Self {
                Self(uuid::Uuid::new_v4().to_string())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

define_id!(UserId);
define_id!(TemplateId);
define_id!(LeadId);
define_id!(QuoteId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_id_new_and_as_str() {
        let id = UserId::new("user-123");
        assert_eq!(id.as_str(), "user-123");
    }

    #[test]
    fn quote_id_generate_produces_unique_values() {
        let id1 = QuoteId::generate();
        let id2 = QuoteId::generate();
        assert_ne!(id1, id2);
    }
}
