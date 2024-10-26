use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Form;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(_form: Form<FormData>) -> impl IntoResponse {
    (StatusCode::OK, "OK")
}
