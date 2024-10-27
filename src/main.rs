use newslatter::configuration::get_configuration;
use newslatter::database::Database;
use newslatter::routes::router::routes;
use std::sync::Arc;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

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
