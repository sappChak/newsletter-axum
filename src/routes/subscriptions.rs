use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Form};
use serde::Deserialize;
use sqlx::types::chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use crate::database::database::Database;

#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(
    Extension(db): Extension<Arc<Database>>,
    Form(form): Form<FormData>,
) -> impl IntoResponse {
    if form.name.is_empty() || form.email.is_empty() {
        return (StatusCode::BAD_REQUEST, "Missing name or email");
    }

    let _ = sqlx::query!(
        r#"
          INSERT INTO subscriptions (id, email, name, subscribed_at)
          VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(&db.pool)
    .await
    .unwrap();

    (StatusCode::OK, "OK")
}
