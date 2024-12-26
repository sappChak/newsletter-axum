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

    let response = app.get("/subscriptions/confirm").await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called(pool: PgPool) {
    let app = spawn_test_app(pool).await.unwrap();
    let form_data = "name=Andrii%20Konotop&email=aws.test.receiver@gmail.com";

    let _ = app.post("/subscriptions", form_data).await;

    let confirmation_link = &app.get_confirmation_links().plain_text;
    assert_eq!(confirmation_link.path(), "/subscriptions/confirm");

    let response = app.get(confirmation_link.as_str()).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[sqlx::test]
async fn clicking_on_the_confirmation_link_confirms_a_subscriber(pool: PgPool) {
    // Arrange
    let app = spawn_test_app(pool).await.unwrap();
    let form_data = "name=Andrii%20Konotop&email=aws.test.receiver@gmail.com";

    // Act
    let _ = app.post("/subscriptions", form_data).await;

    let confirmation_link = &app.get_confirmation_links().plain_text;
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
