use crate::Database;
use axum::Router;
use newslatter::routes::router::routes;
use newslatter::telemetry::get_subscriber;
use newslatter::telemetry::init_subscriber;
use once_cell::sync::Lazy;
use sqlx::PgPool;
use std::sync::Arc;

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_span_name = "test".to_string();
    let default_filter_level = "info".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(default_span_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(default_span_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
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
