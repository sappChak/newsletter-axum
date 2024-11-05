use newslatter::configuration::aws_credentials::StaticCredentials;
use newslatter::configuration::config::get_configuration;
use newslatter::database::db::Database;
use newslatter::domain::SubscriberEmail;
use newslatter::email_client::SESWorkflow;
use newslatter::routes::router::routes;
use newslatter::telemetry::{get_subscriber, init_subscriber};

use aws_config::Region;
use aws_sdk_sesv2::config::SharedCredentialsProvider;
use aws_sdk_sesv2::Client;

use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let configuration = get_configuration().expect("Failed to read configuration.");

    let subscriber = get_subscriber(
        "newslatter".to_string(),
        "info".to_string(),
        std::io::stdout,
    );
    init_subscriber(subscriber);

    let shared_config = aws_config::SdkConfig::builder()
        .region(Region::new(configuration.aws.region))
        .credentials_provider(SharedCredentialsProvider::new(StaticCredentials::new(
            configuration.aws.access_key_id,
            configuration.aws.secret_access_key,
        )))
        .build();

    let client = Client::new(&shared_config);
    let aws_client = Arc::new(SESWorkflow::new(
        client,
        SubscriberEmail::parse("aws.test.sender@gmail.com".to_string())?,
    ));

    let db = Arc::new(Database::new(configuration.database.with_db()).await?);

    let app = routes(db, aws_client);

    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    ))
    .await
    .unwrap_or_else(|_| {
        eprintln!(
            "failed to bind to address: {}:{}",
            configuration.application.host, configuration.application.port
        );
        std::process::exit(1);
    });
    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
    Ok(())
}
