use anyhow::anyhow;
use aws_sdk_sesv2::types::Body;
use aws_sdk_sesv2::types::Content;
use aws_sdk_sesv2::types::Destination;
use aws_sdk_sesv2::types::EmailContent;
use aws_sdk_sesv2::types::Message;
use aws_sdk_sesv2::Client;

use crate::domain::SubscriberEmail;

pub struct SESWorkflow {
    aws_client: Client,
    // Sender
    verified_email: SubscriberEmail,
}

impl SESWorkflow {
    pub fn new(aws_client: Client, verified_email: String) -> Self {
        Self {
            aws_client,
            verified_email: SubscriberEmail::parse(verified_email).expect("Bla bla bla"),
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), anyhow::Error> {
        let email_content = EmailContent::builder()
            .simple(
                Message::builder()
                    .subject(Content::builder().data(subject).build()?)
                    .body(
                        Body::builder()
                            .html(Content::builder().data(html_content).build()?)
                            .text(Content::builder().data(text_content).build()?)
                            .build(),
                    )
                    .build(),
            )
            .build();

        let res = self
            .aws_client
            .send_email()
            .from_email_address(self.verified_email.as_ref())
            .destination(
                Destination::builder()
                    .to_addresses(recipient.as_ref())
                    .build(),
            )
            .content(email_content);

        match res.send().await {
            Ok(output) => {
                if let Some(message_id) = output.message_id {
                    tracing::info!("Message sent: {}", message_id);
                    Ok(())
                } else {
                    Err(anyhow!("Message sent, but no message ID was returned"))
                }
            }
            Err(e) => Err(anyhow!(
                "Error sending welcome email to {}: {}",
                recipient.as_ref(),
                e
            )),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::email_client::SESOptions;
//     use crate::{domain::SubscriberEmail, email_client::SESWorkflow};
//
//     use fake::faker::lorem::en::{Paragraph, Sentence};
//     use fake::{faker::internet::en::SafeEmail, Fake};
//     use wiremock::matchers::any;
//     use wiremock::{Mock, MockServer, ResponseTemplate};
//
//     #[tokio::test]
//     async fn send_email_fires_a_request_to_base_url() {
//         let mock_server = MockServer::start().await;
//         let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
//
//         Mock::given(any())
//             .respond_with(ResponseTemplate::new(200))
//             .expect(1)
//             .mount(&mock_server)
//             .await;
//
//         let email_client = SESWorkflow::new(email_client_options);
//
//         let recipient = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
//         let subject: String = Sentence(1..2).fake();
//         let html_content: String = Paragraph(1..10).fake();
//         let text_content: String = Paragraph(1..10).fake();
//
//         let _ = email_client
//             .send_email(recipient, &subject, &html_content, &text_content)
//             .await;
//
//         mock_server.verify().await;
//     }
// }
