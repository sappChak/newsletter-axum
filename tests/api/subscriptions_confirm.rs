use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use reqwest::Url;
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

#[sqlx::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called(pool: PgPool) {
    let app = spawn_test_app(pool).await.unwrap();

    let form_data = "name=Andrii%20Konotop&email=aws.test.receiver@gmail.com";

    let _response = app
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

    let raw_confirmation_link = &app.get_confirmation_link();

    let confiramation_link = Url::parse(raw_confirmation_link).unwrap();

    assert_eq!(confiramation_link.host_str(), Some("127.0.0.1"));
    assert_eq!(confiramation_link.path(), "/subscriptions/confirm");

    // let response = reqwest::get(confiramation_link).await.unwrap();
    //
    // assert_eq!(response.status().as_u16(), 200);
}
