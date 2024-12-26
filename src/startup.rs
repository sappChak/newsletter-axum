use aws_config::Region;
use aws_sdk_sesv2::config::SharedCredentialsProvider;
use aws_sdk_sesv2::Client;

use crate::configuration::aws_credentials::StaticCredentials;
use crate::configuration::config::Configuration;
use crate::telemetry::{get_subscriber, init_subscriber};

pub fn configure_sdk_config(
    configuration: &Configuration,
) -> Result<aws_config::SdkConfig, anyhow::Error> {
    let region = Region::new(configuration.aws.region.clone());

    let credentials_provider = SharedCredentialsProvider::new(StaticCredentials::new(
        configuration.aws.access_key_id.clone(),
        configuration.aws.secret_access_key.clone(),
    ));

    let sdk_config = aws_config::SdkConfig::builder()
        .region(region)
        .credentials_provider(credentials_provider)
        .build();

    Ok(sdk_config)
}

pub fn create_aws_client(sdk_config: &aws_config::SdkConfig) -> Result<Client, anyhow::Error> {
    let aws_client = Client::new(sdk_config);
    Ok(aws_client)
}

pub fn init_logging(configuration: &Configuration) -> Result<(), anyhow::Error> {
    let subscriber = get_subscriber(
        configuration.application.logger_name.clone(),
        configuration.application.default_env_filter.clone(),
        std::io::stdout,
    );
    init_subscriber(subscriber);
    Ok(())
}

pub async fn start_server(
    configuration: &Configuration,
    app: axum::Router,
) -> Result<(), anyhow::Error> {
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
