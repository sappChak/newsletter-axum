use crate::db::database::Database;
use crate::routes::health_check;
use crate::routes::subscribe;
use axum::routing::get;
use axum::routing::post;
use axum::Extension;
use axum::Router;
use std::sync::Arc;

use tower_http::trace;
use tower_http::trace::TraceLayer;
use tracing::Level;

pub fn routes(state: Arc<Database>) -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .layer(Extension(state))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
}
