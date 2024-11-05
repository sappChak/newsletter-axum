use crate::Database;
use newslatter::configuration::aws_credentials::StaticCredentials;
use newslatter::configuration::config::get_configuration;
use newslatter::domain::SubscriberEmail;
use newslatter::email_client::SESWorkflow;
use newslatter::routes::router::routes;
use newslatter::telemetry::get_subscriber;
use newslatter::telemetry::init_subscriber;

use aws_config::Region;
use aws_sdk_sesv2::config::SharedCredentialsProvider;
use aws_sdk_sesv2::Client;
use axum::Router;
use once_cell::sync::Lazy;
use sqlx::PgPool;

use std::sync::Arc;

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

pub struct TestApp {
    pub db_state: Arc<Database>,
    pub router: Router,
}

pub async fn spawn_test_app(pool: PgPool) -> Result<TestApp, Box<dyn std::error::Error>> {
    Lazy::force(&TRACING);

    let configuration = get_configuration().expect("Failed to read configuration.");

    let shared_config = aws_config::SdkConfig::builder()
        .region(Region::new(configuration.aws.region))
        .credentials_provider(SharedCredentialsProvider::new(StaticCredentials::new(
            configuration.aws.access_key_id,
            configuration.aws.secret_access_key,
        )))
        .build();

    let db_state = Arc::new(Database { pool });
    let client = Client::new(&shared_config);

    let client_state = Arc::new(SESWorkflow::new(
        client,
        SubscriberEmail::parse("aws.test.sender@gmail.com".to_string())?,
    ));

    let router = routes(db_state.clone(), client_state.clone());

    Ok(TestApp { db_state, router })
}
