use axum::{
    http::Request,
    routing::{get, post},
    Extension, Router,
};
use tower_http::trace::TraceLayer;
use tracing::Level;

use std::sync::Arc;

use super::newsletter::publish_newsletter;
use crate::{
    routes::{health_check, subscribe, subscriptions_confirm::confirm},
    state::AppState,
};

pub fn router(state: AppState, base_url: Arc<String>) -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .route("/subscriptions/confirm", get(confirm))
        .route("/newsletters", post(publish_newsletter))
        .with_state(state)
        .layer(Extension(base_url))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                let request_id = uuid::Uuid::new_v4();
                tracing::span!(
                    Level::DEBUG,
                    "request",
                    %request_id,
                    method = ?request.method(),
                    uri = %request.uri(),
                    version = ?request.version(),
                )
            }),
        )
}
