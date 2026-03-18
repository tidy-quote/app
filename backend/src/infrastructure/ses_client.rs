use async_trait::async_trait;
use aws_sdk_sesv2::types::{Body, Content, Destination, EmailContent, Message};
use aws_sdk_sesv2::Client;

use crate::application::ports::{EmailError, EmailSender};

pub struct SesEmailClient {
    client: Client,
    sender: String,
}

impl SesEmailClient {
    pub async fn new(sender: String) -> Self {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let client = Client::new(&config);
        Self { client, sender }
    }
}

#[async_trait]
impl EmailSender for SesEmailClient {
    async fn send_email(&self, to: &str, subject: &str, html_body: &str) -> Result<(), EmailError> {
        let dest = Destination::builder().to_addresses(to).build();

        let subject_content = Content::builder().data(subject).charset("UTF-8").build().map_err(|e| EmailError::SendFailed(e.to_string()))?;
        let body_content = Content::builder().data(html_body).charset("UTF-8").build().map_err(|e| EmailError::SendFailed(e.to_string()))?;
        let body = Body::builder().html(body_content).build();
        let message = Message::builder()
            .subject(subject_content)
            .body(body)
            .build();

        let email_content = EmailContent::builder().simple(message).build();

        self.client
            .send_email()
            .from_email_address(&self.sender)
            .destination(dest)
            .content(email_content)
            .send()
            .await
            .map_err(|e| EmailError::SendFailed(e.to_string()))?;

        Ok(())
    }
}
