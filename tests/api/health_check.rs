use axum::{body::Body, extract::Request, http::StatusCode};
use sqlx::PgPool;
use tower::util::ServiceExt;

use crate::helpers::{mock_aws_sesv2, spawn_test_app};

#[sqlx::test]
async fn health_check_works(pool: PgPool) {
    let aws_client = mock_aws_sesv2();
    let app = spawn_test_app(pool, aws_client).await.unwrap();

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
