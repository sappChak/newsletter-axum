use crate::database::database::Database;
use crate::routes::health_check;
use crate::routes::subscribe;
use axum::routing::get;
use axum::routing::post;
use axum::Extension;
use axum::Router;
use std::sync::Arc;

pub fn routes(state: Arc<Database>) -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .layer(Extension(state))
}
