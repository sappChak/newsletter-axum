use axum::{body::Body, extract::Request, http::StatusCode};
use sqlx::PgPool;
use tower::util::ServiceExt;

mod common;
use common::spawn_test_app;
use newsletter::database::db::Database;

#[sqlx::test]
async fn health_check_works(pool: PgPool) {
    let app = spawn_test_app(pool).await.unwrap();

    let response = app
        .router
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
