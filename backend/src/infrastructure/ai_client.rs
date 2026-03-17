use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::application::ports::{AiClient, AiError};
use crate::domain::entities::*;

pub struct AiClientConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

pub struct OpenAiCompatibleClient {
    config: AiClientConfig,
    http: Client,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f64,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: Vec<ContentPart>,
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrlContent },
}

#[derive(Serialize)]
struct ImageUrlContent {
    url: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
}

#[derive(Deserialize)]
struct ChatResponseMessage {
    content: String,
}

const TEMPERATURE: f64 = 0.3;

fn extract_json(text: &str) -> &str {
    let trimmed = text.trim();
    if let Some(start) = trimmed.find("```") {
        let after_backticks = &trimmed[start + 3..];
        let content = after_backticks
            .strip_prefix("json")
            .unwrap_or(after_backticks);
        if let Some(end) = content.find("```") {
            return content[..end].trim();
        }
    }
    trimmed
}

impl OpenAiCompatibleClient {
    pub fn new(config: AiClientConfig) -> Self {
        Self {
            config,
            http: Client::new(),
        }
    }

    async fn chat_completion(&self, messages: Vec<ChatMessage>) -> Result<String, AiError> {
        let request = ChatRequest {
            model: self.config.model.clone(),
            messages,
            temperature: TEMPERATURE,
        };

        let response = self
            .http
            .post(format!("{}/chat/completions", self.config.base_url))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AiError::RequestFailed(e.to_string()))?;

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(AiError::RateLimited);
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AiError::RequestFailed(format!("HTTP {}: {}", status, body)));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| AiError::ParseError(e.to_string()))?;

        chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| AiError::ParseError("empty response from AI".to_string()))
    }

    fn build_lead_content(lead: &Lead) -> Vec<ContentPart> {
        let mut parts = Vec::new();

        if let Some(ref text) = lead.raw_text {
            parts.push(ContentPart::Text { text: text.clone() });
        }

        for image in &lead.image_data {
            let url = if image.starts_with("data:") {
                image.clone()
            } else {
                format!("data:image/jpeg;base64,{}", image)
            };
            parts.push(ContentPart::ImageUrl {
                image_url: ImageUrlContent { url },
            });
        }

        if parts.is_empty() {
            parts.push(ContentPart::Text {
                text: "(no content provided)".to_string(),
            });
        }

        parts
    }
}

#[async_trait]
impl AiClient for OpenAiCompatibleClient {
    async fn extract_job_details(
        &self,
        lead: &Lead,
        template: &PricingTemplate,
    ) -> Result<JobSummary, AiError> {
        let categories_json = serde_json::to_string(&template.categories).unwrap_or_default();

        let system_prompt = format!(
            r#"You are a job detail extractor for a service business. Analyze the customer's message and/or images and extract structured job details.

Available service categories: {}

Respond with valid JSON matching this schema:
{{
  "serviceType": "string (must match a category name)",
  "propertySize": "string or null",
  "requestedDate": "string or null",
  "requestedTime": "string or null",
  "missingInfo": ["list of things you need clarified"],
  "extractedDetails": {{"key": "value pairs of any other relevant details"}}
}}"#,
            categories_json
        );

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: vec![ContentPart::Text {
                    text: system_prompt,
                }],
            },
            ChatMessage {
                role: "user".to_string(),
                content: Self::build_lead_content(lead),
            },
        ];

        let response_text = self.chat_completion(messages).await?;
        let json_text = extract_json(&response_text);

        serde_json::from_str(json_text)
            .map_err(|e| AiError::ParseError(format!("failed to parse job summary: {}", e)))
    }

    async fn generate_follow_up(
        &self,
        summary: &JobSummary,
        quote: &QuoteDraft,
        tone: &ToneOption,
        currency: &str,
    ) -> Result<String, AiError> {
        let tone_instruction = match tone {
            ToneOption::Friendly => "Use a warm, friendly, and approachable tone.",
            ToneOption::Direct => "Be concise and professional. Get straight to the point.",
            ToneOption::Premium => {
                "Use a polished, premium tone that conveys expertise and quality."
            }
        };

        let system_prompt = format!(
            r#"You are a follow-up message writer for a service business.
{}
Write a short follow-up message to send to the customer based on the job details and quote.
Include the estimated price and a brief summary. Keep it under 200 words."#,
            tone_instruction
        );

        let user_content = format!(
            "Job summary: {}\nEstimated price: {:.2} {}\nPrice breakdown: {}",
            serde_json::to_string(summary).unwrap_or_default(),
            quote.estimated_price,
            currency,
            serde_json::to_string(&quote.price_breakdown).unwrap_or_default()
        );

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: vec![ContentPart::Text {
                    text: system_prompt,
                }],
            },
            ChatMessage {
                role: "user".to_string(),
                content: vec![ContentPart::Text { text: user_content }],
            },
        ];

        self.chat_completion(messages).await
    }
}
