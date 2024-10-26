use axum::{
    routing::{get, post},
    Router,
};
use newslatter::configuration::get_configuration;
use newslatter::{
    app_state::AppStateInner,
    routes::{health_check, subscribe},
};
use std::sync::Arc;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn app(state: Arc<AppStateInner>) -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(state)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_string = configuration.database.connection_string();

    let state = Arc::new(AppStateInner::new(&connection_string).await);

    let app = app(state);

    let listener =
        tokio::net::TcpListener::bind(format!("0.0.0.0:{}", configuration.application_port))
            .await
            .unwrap_or_else(|_| {
                eprintln!("failed to bind to port {}", configuration.application_port);
                std::process::exit(1);
            });

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
