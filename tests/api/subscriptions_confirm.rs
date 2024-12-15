use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sqlx::PgPool;
use tower::util::ServiceExt;

use crate::helpers::spawn_test_app;

#[sqlx::test]
async fn confirmations_without_token_are_rejected_with_a_400(pool: PgPool) {
    // Arrange
    let app = spawn_test_app(pool).await.unwrap();

    // Act
    let response = app
        .router
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/subscriptions/confirm")
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
