use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sqlx::PgPool;
use tower::util::ServiceExt;

use crate::helpers::spawn_test_app;

#[sqlx::test]
async fn confirmations_without_token_are_rejected_with_a_400(pool: PgPool) {
    let app = spawn_test_app(pool).await.unwrap();

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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called(pool: PgPool) {
    let app = spawn_test_app(pool).await.unwrap();
    let form_data = "name=Andrii%20Konotop&email=aws.test.receiver@gmail.com";

    let _ = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/subscriptions")
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap(),
        )
        .await
        .unwrap();

    let confirmation_link = &app.get_confirmation_links().plain_text;
    assert_eq!(confirmation_link.path(), "/subscriptions/confirm");

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(confirmation_link.as_str())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 200);
}
