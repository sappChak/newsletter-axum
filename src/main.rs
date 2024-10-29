use newslatter::config::configuration::get_configuration;
use newslatter::db::database::Database;
use newslatter::routes::router::routes;
use std::sync::Arc;
use tracing::dispatcher::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    LogTracer::init().expect("Failed to set logger");

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new("newslatter".into(), std::io::stdout);

    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
        .into();

    set_global_default(subscriber).expect("Failed to set subscriber");

    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_string = configuration.database.connection_string();

    let state = Arc::new(Database::new(&connection_string).await?);
    let app = routes(state);

    let listener =
        tokio::net::TcpListener::bind(format!("0.0.0.0:{}", configuration.application_port))
            .await
            .unwrap_or_else(|_| {
                eprintln!("failed to bind to port {}", configuration.application_port);
                std::process::exit(1);
            });

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
