use axum::{body::Body, extract::Request, http::StatusCode};
use newslatter::startup::routes;
use tower::ServiceExt;

#[tokio::test]
async fn health_check_works() {
    let app = routes();

    let response = app
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
