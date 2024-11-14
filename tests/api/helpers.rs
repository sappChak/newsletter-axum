use aws_sdk_sesv2::operation::send_email::SendEmailOutput;
use aws_sdk_sesv2::Client;
use aws_smithy_mocks_experimental::mock;
use aws_smithy_mocks_experimental::mock_client;
use aws_smithy_mocks_experimental::RuleMode;
use axum::Router;
use once_cell::sync::Lazy;
use sqlx::PgPool;

use newsletter::configuration::config::get_configuration;
use newsletter::database::db::Database;
use newsletter::routes::router::routes;
use newsletter::ses_workflow::SESWorkflow;
use newsletter::telemetry::get_subscriber;
use newsletter::telemetry::init_subscriber;

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

pub async fn mock_aws_sesv2_client() -> Client {
    let mock_send_email = mock!(Client::send_email).then_output(|| {
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

    let db_state = Arc::new(Database { pool });
    let ses_state = Arc::new(SESWorkflow::new(
        mock_aws_sesv2_client().await,
        configuration.aws.verified_email.clone(),
    ));

    let router = routes(db_state.clone(), ses_state.clone());

    Ok(TestApp { db_state, router })
}
