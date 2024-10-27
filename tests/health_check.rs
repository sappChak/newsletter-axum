use axum::{body::Body, extract::Request, http::StatusCode};
use newslatter::configuration::configuration::get_configuration;
use newslatter::database::database::Database;
use newslatter::routes::router::routes;
use std::sync::Arc;
use tower::util::ServiceExt;

#[tokio::test]
async fn health_check_works() {
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();

    let state = Arc::new(Database::new(&connection_string).await.unwrap());
    let routes = routes(state);

    let response = routes
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
