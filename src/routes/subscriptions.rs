use crate::db::database::Database;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Form};
use serde::Deserialize;
use sqlx::types::chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(
    Extension(db): Extension<Arc<Database>>,
    Form(form): Form<FormData>,
) -> impl IntoResponse {
    match sqlx::query!(
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
    {
        Ok(_) => {
            tracing::info!("Subscriber was saved successfully");
            StatusCode::OK
        }
        Err(e) => {
            tracing::error!("Error executing SQL query: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
