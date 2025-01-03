use std::sync::{Arc, RwLock};

use aws_sdk_sesv2::{operation::send_email::SendEmailOutput, Client};
use aws_smithy_mocks_experimental::{mock, mock_client, RuleMode};
use axum::{
    body::Body,
    http::{self, Request},
    response::Response,
    Router,
};
use once_cell::sync::Lazy;
use serde_json::to_string;
use sqlx::PgPool;
use tower::ServiceExt;

use newsletter::{
    configuration::config::get_configuration,
    database::db::Database,
    routes::router::router,
    ses_workflow::SESWorkflow,
    state::AppState,
    telemetry::{get_subscriber, init_subscriber},
};

pub type CapturedRequestContent = Arc<RwLock<Option<(String, String)>>>;

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
}

impl TestApp {
    pub async fn get(&self, uri: &str) -> Response {
        self.router
            .clone()
            .oneshot(
                Request::builder()
                    .method(http::Method::GET)
                    .uri(uri)
                    .header(
                        http::header::CONTENT_TYPE,
                        "application/x-www-form-urlencoded",
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    pub async fn post(&self, uri: &str, form_data: &'static str) -> Response {
        self.router
            .clone()
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri(uri)
                    .header(
                        http::header::CONTENT_TYPE,
                        "application/x-www-form-urlencoded",
                    )
                    .body(Body::from(form_data))
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    pub async fn post_json(&self, uri: &str, body: impl serde::Serialize) -> Response {
        let json_body = to_string(&body).expect("Failed to convert serialized JSON to string");
        self.router
            .clone()
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri(uri)
                    .header(http::header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json_body))
                    .unwrap(),
            )
            .await
            .unwrap()
    }
}

pub fn get_confirmation_links(
    captured_request_content: CapturedRequestContent,
) -> ConfirmationLinks {
    let content = captured_request_content.read().unwrap().clone().unwrap();
    ConfirmationLinks {
        html: extract_link(&content.0),
        plain_text: extract_link(&content.1),
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

pub fn mock_aws_sesv2_with_request_capture(
    captured_request_content: CapturedRequestContent,
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

                *captured_request_content.write().unwrap() = Some((html_body, text_body));
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

    mock_client!(aws_sdk_sesv2, RuleMode::MatchAny, [&mock_send_email])
}

pub fn mock_aws_sesv2() -> Client {
    let mock_send_email = mock!(Client::send_email).then_output(|| {
        SendEmailOutput::builder()
            .message_id("newsletter-email")
            .build()
    });
    mock_client!(aws_sdk_sesv2, RuleMode::Sequential, [&mock_send_email])
}

pub fn mock_aws_sesv2_no_requests() -> Client {
    let mock_send_email = mock!(Client::send_email)
        .match_requests(|_| {
            // Fail the test if any request is made
            panic!("Unexpected request to AWS SES");
        })
        .then_output(|| {
            // This output will never be used, as the test will fail
            panic!("This output should never be produced");
        });
    mock_client!(aws_sdk_sesv2, RuleMode::Sequential, [&mock_send_email])
}

pub async fn spawn_test_app(pool: PgPool, client: Client) -> Result<TestApp, anyhow::Error> {
    Lazy::force(&TRACING);

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        c.aws.verified_email = "sender@example.com".to_string();
        c
    };

    let db = Arc::new(Database { pool });
    let ses = Arc::new(SESWorkflow::new(client, configuration.aws.verified_email));
    let base_url = Arc::new(configuration.application.base_url);

    let state = AppState {
        db: db.clone(),
        workflow: ses.clone(),
    };

    let router = router(state, base_url.clone());

    Ok(TestApp { db, router })
}
