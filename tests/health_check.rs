use std::sync::Arc;

use axum::{body::Body, extract::Request, http::StatusCode};
use newslatter::{configuration::get_configuration, database::Database, routes::router::routes};
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
