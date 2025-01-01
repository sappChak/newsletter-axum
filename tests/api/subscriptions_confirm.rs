use std::sync::{Arc, RwLock};

use axum::http::StatusCode;
use sqlx::PgPool;

use crate::helpers::{
    get_confirmation_links, mock_aws_sesv2, mock_aws_sesv2_with_request_capture, spawn_test_app,
};

#[sqlx::test]
async fn confirmations_without_token_are_rejected_with_a_400(pool: PgPool) {
    // Arrange
    let client = mock_aws_sesv2();
    let app = spawn_test_app(pool, client).await.unwrap();

    // Act
    let response = app.get("/subscriptions/confirm").await;

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called(pool: PgPool) {
    // Arrange
    let captured_request_content = Arc::new(RwLock::new(None));
    let client = mock_aws_sesv2_with_request_capture(captured_request_content.clone());
    let app = spawn_test_app(pool, client).await.unwrap();

    let form_data = "name=Andrii%20Konotop&email=aws.test.receiver@gmail.com";

    // Act
    let _ = app.post("/subscriptions", form_data).await;
    let confirmation_link = get_confirmation_links(captured_request_content.clone()).plain_text;

    // Assert
    assert_eq!(confirmation_link.path(), "/subscriptions/confirm");

    let response = app.get(confirmation_link.as_str()).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[sqlx::test]
async fn clicking_on_the_confirmation_link_confirms_a_subscriber(pool: PgPool) {
    // Arrange
    let captured_request_content = Arc::new(RwLock::new(None));
    let aws_client = mock_aws_sesv2_with_request_capture(captured_request_content.clone());
    let app = spawn_test_app(pool, aws_client).await.unwrap();
    let form_data = "name=Andrii%20Konotop&email=aws.test.receiver@gmail.com";

    // Act
    let _ = app.post("/subscriptions", form_data).await;

    let confirmation_link = get_confirmation_links(captured_request_content.clone()).plain_text;
    let _response = app.get(confirmation_link.as_str()).await;

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&app.db.pool)
        .await
        .expect("Failed to fetch saved subscription.");

    // Assert
    assert_eq!(saved.name, "Andrii Konotop");
    assert_eq!(saved.email, "aws.test.receiver@gmail.com");
    assert_eq!(saved.status, "confirmed");
}
