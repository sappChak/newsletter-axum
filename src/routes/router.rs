use axum::http::Request;
use axum::routing::get;
use axum::routing::post;
use axum::Extension;
use axum::Router;
use tower_http::trace::TraceLayer;
use tracing::Level;

use std::sync::Arc;

use crate::database::db::Database;
use crate::email_client::SESWorkflow;
use crate::routes::health_check;
use crate::routes::subscribe;

pub fn routes(db: Arc<Database>, client: Arc<SESWorkflow>) -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .layer(Extension(db))
        .layer(Extension(client))
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
