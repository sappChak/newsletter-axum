use axum::extract::Query;
use axum::{http::StatusCode, response::IntoResponse};

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters))]
pub async fn confirm(parameters: Query<Parameters>) -> impl IntoResponse {
    StatusCode::OK
}
