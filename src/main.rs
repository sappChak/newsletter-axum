use newslatter::config::configuration::get_configuration;
use newslatter::database::db::Database;
use newslatter::routes::router::routes;
use newslatter::telemetry::{get_subscriber, init_subscriber};
use secrecy::ExposeSecret;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = get_subscriber(
        "newslatter".to_string(),
        "info".to_string(),
        std::io::stdout,
    );
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_string = configuration.database.connection_string();

    let state = Arc::new(Database::new(connection_string.expose_secret()).await?);
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
