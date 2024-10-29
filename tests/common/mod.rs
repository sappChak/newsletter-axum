use axum::Router;
use newslatter::routes::router::routes;
use newslatter::telemetry::get_subscriber;
use newslatter::telemetry::init_subscriber;

use crate::Database;

use once_cell::sync::Lazy;
use sqlx::PgPool;
use std::sync::Arc;

static TRACING: Lazy<()> = Lazy::new(|| {
    let subscriber = get_subscriber("test".to_string(), "debug".to_string());
    init_subscriber(subscriber);
});

pub struct TestApp {
    pub state: Arc<Database>,
    pub router: Router,
}

pub async fn spawn_test_app(pool: PgPool) -> TestApp {
    Lazy::force(&TRACING);

    let state = Arc::new(Database { pool });
    let router = routes(state.clone());

    TestApp { state, router }
}
