pub mod health_check;
pub mod subscriptions;

pub use health_check::*;
pub use subscriptions::*;

use crate::db::database::Database;
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
