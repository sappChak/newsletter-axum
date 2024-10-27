use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use newslatter::config::configuration::get_configuration;
use newslatter::db::database::Database;
use newslatter::routes::router::routes;
use std::sync::Arc;
use tower::ServiceExt; // for `call`, `oneshot`, and `ready`

#[tokio::test]
async fn subscribe_returs_200_for_valid_form_data() {
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();

    let state = Arc::new(Database::new(&connection_string).await.unwrap());
    let routes = routes(state.clone());

    let form_data = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = routes
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

    assert_eq!(response.status(), StatusCode::OK);

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&state.pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returs_400_for_data_is_missing() {
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();

    let state = Arc::new(Database::new(&connection_string).await.unwrap());
    let routes = routes(state);

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (form_data, error_message) in test_cases {
        let response = routes
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

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );

        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert!(body.is_empty());
    }
}
