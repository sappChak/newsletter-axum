use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sqlx::PgPool;
use tower::util::ServiceExt;

use crate::helpers::spawn_test_app;

#[sqlx::test]
async fn subscribe_returs_200_for_valid_form_data(pool: PgPool) {
    // Arrange
    let app = spawn_test_app(pool).await.unwrap();
    let form_data = "name=Andrii%20Konotop&email=aws.test.receiver@gmail.com";

    // Act
    let response = app
        .router
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

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test]
async fn subscribe_persists_the_new_subscriber(pool: PgPool) {
    // Arrange
    let app = spawn_test_app(pool).await.unwrap();
    let form_data = "name=Andrii%20Konotop&email=aws.test.receiver@gmail.com";

    // Act
    let _ = app
        .router
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

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&app.db.pool)
        .await
        .expect("Failed to fetch saved subscription.");

    // Assert
    assert_eq!(saved.name, "Andrii Konotop");
    assert_eq!(saved.email, "aws.test.receiver@gmail.com");
    assert_eq!(saved.status, "pending_confirmation");
}

#[sqlx::test]
async fn subscribe_returs_422_for_data_is_missing(pool: PgPool) {
    // Arrange
    let app = spawn_test_app(pool).await.unwrap();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    // Act
    for (form_data, error_message) in test_cases {
        let response = app
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

        // Assert
        assert_eq!(
            response.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "The API did not fail with 422 UNPROCESSABLE_ENTITY when the payload was {}.",
            error_message
        );
    }
}

#[sqlx::test]
async fn subscribe_returs_400_when_fields_are_present_but_invalid(pool: PgPool) {
    // Arrange
    let app = spawn_test_app(pool).await.unwrap();
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];

    // Act
    for (form_data, error_message) in test_cases {
        let response = app
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

        // Assert
        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "The API did not fail with 400 BAD_REQUEST when the payload was {}.",
            error_message
        );
    }
}

#[sqlx::test]
async fn subscribe_sends_a_confirmation_email_with_a_link(pool: PgPool) {
    // Arrange
    let app = spawn_test_app(pool).await.unwrap();
    let form_data = "name=Andrii%20Konotop&email=aws.test.receiver@gmail.com";

    // Act
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

    // Assert
    let confirmation_links = &app.get_confirmation_links();
    assert_eq!(confirmation_links.html, confirmation_links.plain_text);
}

#[sqlx::test]
async fn subscribe_fails_if_there_is_a_fatal_database_error(pool: PgPool) {
    // Arrange
    let app = spawn_test_app(pool).await.unwrap();
    let form_data = "name=Andrii%20Konotop&email=aws.test.receiver@gmail.com";

    // Act
    sqlx::query!("ALTER TABLE subscription_tokens DROP COLUMN subscription_token;")
        .execute(&app.db.pool)
        .await
        .expect("Failed to drop subscriptions table.");

    let response = app
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

    // Assert
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
