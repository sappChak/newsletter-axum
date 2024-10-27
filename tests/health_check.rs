use axum::{body::Body, extract::Request, http::StatusCode};
use newslatter::db::database::Database;
use newslatter::routes::routes;
use sqlx::PgPool;
use std::sync::Arc;
use tower::util::ServiceExt;

#[sqlx::test]
async fn health_check_works(pool: PgPool) {
    let state = Arc::new(Database { pool });
    let routes = routes(state.clone());

    let response = routes
        .oneshot(
            Request::builder()
                .uri("/health_check")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
