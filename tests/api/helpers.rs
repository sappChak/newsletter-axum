use std::sync::Arc;
use std::sync::Mutex;

use aws_sdk_sesv2::operation::send_email::SendEmailOutput;
use aws_sdk_sesv2::Client;
use aws_smithy_mocks_experimental::{mock, mock_client, RuleMode};
use axum::Router;
use once_cell::sync::Lazy;
use sqlx::PgPool;

use newsletter::configuration::config::get_configuration;
use newsletter::database::db::Database;
use newsletter::routes::router::router;
use newsletter::ses_workflow::SESWorkflow;
use newsletter::telemetry::get_subscriber;
use newsletter::telemetry::init_subscriber;

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_span_name = "test".to_string();
    let default_filter_level = "info".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(default_span_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        // Send all logs into void ----------------------------------------------------!
        let subscriber = get_subscriber(default_span_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

pub struct TestApp {
    pub db: Arc<Database>,
    pub router: Router,
    pub captured_request_content: Arc<Mutex<Option<(String, String)>>>,
}

impl TestApp {
    pub fn get_confirmation_links(&self) -> ConfirmationLinks {
        let content = self
            .captured_request_content
            .lock()
            .unwrap()
            .clone()
            .unwrap();

        ConfirmationLinks {
            html: extract_link(&content.0),
            plain_text: extract_link(&content.1),
        }
    }
}

pub fn extract_link(s: &str) -> reqwest::Url {
    let links: Vec<_> = linkify::LinkFinder::new()
        .links(s)
        .filter(|l| *l.kind() == linkify::LinkKind::Url)
        .collect();
    let raw_link = links[0].as_str().to_owned();
    let confiramation_link = reqwest::Url::parse(&raw_link).unwrap();
    assert_eq!(confiramation_link.host_str(), Some("127.0.0.1"));

    confiramation_link
}

pub async fn mock_aws_sesv2_client(
    captured_request_content: Arc<Mutex<Option<(String, String)>>>,
) -> Client {
    let mock_send_email = mock!(Client::send_email)
        .match_requests(move |req| {
            let destination = req.destination().unwrap();
            let content = req.content().unwrap();

            let matches_recipient = destination
                .to_addresses()
                .contains(&"aws.test.receiver@gmail.com".into());

            if matches_recipient {
                let html_body = content
                    .simple()
                    .unwrap()
                    .body()
                    .unwrap()
                    .html()
                    .unwrap()
                    .data()
                    .to_string();

                let text_body = content
                    .simple()
                    .unwrap()
                    .body()
                    .unwrap()
                    .text()
                    .unwrap()
                    .data()
                    .to_string();

                *captured_request_content.lock().unwrap() = Some((html_body, text_body));
                true
            } else {
                false
            }
        })
        .then_output(|| {
            SendEmailOutput::builder()
                .message_id("newsletter-email")
                .build()
        });
    mock_client!(aws_sdk_sesv2, RuleMode::Sequential, [&mock_send_email])
}

pub async fn spawn_test_app(pool: PgPool) -> Result<TestApp, Box<dyn std::error::Error>> {
    Lazy::force(&TRACING);

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        c.aws.verified_email = "sender@example.com".to_string();
        c
    };

    let captured_request_content = Arc::new(Mutex::new(None));

    let aws_client = mock_aws_sesv2_client(captured_request_content.clone()).await;

    let db = Arc::new(Database { pool });
    let ses = Arc::new(SESWorkflow::new(
        aws_client,
        configuration.aws.verified_email.clone(),
    ));
    let base_url = Arc::new(configuration.application.base_url.clone());

    let router = router(db.clone(), ses.clone(), base_url);

    Ok(TestApp {
        db,
        router,
        captured_request_content,
    })
}
