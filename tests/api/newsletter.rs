use std::sync::{Arc, RwLock};

use axum::http::StatusCode;
use sqlx::PgPool;

use crate::helpers::{
    get_confirmation_links, mock_aws_sesv2, mock_aws_sesv2_with_request_capture, spawn_test_app,
    CapturedRequestContent, TestApp,
};

async fn create_uncorfirmed_subscriber(app: &TestApp) {
    let form_data = "name=Andrii%20Konotop&email=aws.test.receiver@gmail.com";
    let _ = app.post("/subscriptions", form_data).await;
}

async fn create_confirmed_subscriber(
    app: &TestApp,
    captured_request_content: CapturedRequestContent,
) {
    create_uncorfirmed_subscriber(app).await;
    let confirmation_link = get_confirmation_links(captured_request_content.clone()).plain_text;
    app.get(confirmation_link.as_str()).await;
}

#[sqlx::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers(pool: PgPool) {
    // Arrange
    let client = mock_aws_sesv2();
    let app = spawn_test_app(pool, client).await.unwrap();
    create_uncorfirmed_subscriber(&app).await;

    let body = serde_json::json!({
    "title": "Newsletter title",
    "content": {
    "text": "Newsletter body as plain text",
    "html": "<p>Newsletter body as HTML</p>",
    }
    });

    // Act
    let response = app.post_json("/newsletters", body).await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);
}

#[sqlx::test]
async fn newsletters_are_delivered_to_confirmed_subscribers(pool: PgPool) {
    // Arrange
    let captured_request_content = Arc::new(RwLock::new(None));
    let client = mock_aws_sesv2_with_request_capture(captured_request_content.clone());

    let app = spawn_test_app(pool, client).await.unwrap();
    create_confirmed_subscriber(&app, captured_request_content.clone()).await;

    let body = serde_json::json!({
    "title": "Newsletter title",
    "content": {
    "text": "Newsletter body as plain text",
    "html": "<p>Newsletter body as HTML</p>",
    }
    });

    // Act
    let response = app.post_json("/newsletters", body).await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);
}

#[sqlx::test]
async fn newsletters_returs_422_for_invalid_data(pool: PgPool) {
    // Arrange
    let client = mock_aws_sesv2();
    let app = spawn_test_app(pool, client).await.unwrap();

    let test_cases = vec![
        (
            serde_json::json!({
            "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>",
            }
            }),
            "missing title",
        ),
        (
            serde_json::json!({"title": "Newsletter!"}),
            "missing content",
        ),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = app.post_json("/newsletters", invalid_body).await;
        // Assert
        assert_eq!(
            response.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "The API did not fail with 400 BAD_REQUEST when the payload was {}.",
            error_message
        );
    }
}
