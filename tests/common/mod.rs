use axum::Router;
use newsletter::startup::configure_aws;
use newsletter::startup::create_aws_client;
use once_cell::sync::Lazy;
use sqlx::PgPool;

use crate::Database;
use newsletter::configuration::config::get_configuration;
use newsletter::email_client::SESWorkflow;
use newsletter::routes::router::routes;
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

pub async fn spawn_test_app(pool: PgPool) -> Result<TestApp, Box<dyn std::error::Error>> {
    Lazy::force(&TRACING);

    let configuration = get_configuration().expect("Failed to read configuration.");

    let shared_config = configure_aws(&configuration)?;
    let aws_client = create_aws_client(&shared_config)?;

    let db_state = Arc::new(Database { pool });
    let ses_state = Arc::new(SESWorkflow::new(
        aws_client,
        configuration.aws.verified_email.clone(),
    ));

    let router = routes(db_state.clone(), ses_state.clone());

    Ok(TestApp { db_state, router })
}
