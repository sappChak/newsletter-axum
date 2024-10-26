use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use newslatter::configuration::get_configuration;
use sqlx::{Connection, PgConnection};
use tower::ServiceExt; // for `call`, `oneshot`, and `ready`

#[tokio::test]
async fn subscribe_returs_200_for_valid_form_data() {
    let app = app();

    let configuration = get_configuration().expect("Failed to read configuration");

    let connection_string = configuration.database.connection_string();

    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres");

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription.");

    let form_data = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = app
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
}

#[tokio::test]
async fn subscribe_returs_400_for_data_is_missing() {
    let app = app();

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (form_data, error_message) in test_cases {
        let response = app
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
