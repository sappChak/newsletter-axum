use reqwest::Client;

use crate::domain::SubscriberEmail;

#[allow(dead_code)]
pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
}

pub struct EmailClientOptions {
    pub base_url: String,
    pub sender: SubscriberEmail,
}

impl EmailClient {
    pub fn new(options: EmailClientOptions) -> Self {
        Self {
            http_client: Client::new(),
            base_url: options.base_url,
            sender: options.sender,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), String> {
        todo!()
    }
}
