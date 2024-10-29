use crate::db::database::Database;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Form};
use serde::Deserialize;
use sqlx::types::chrono::Utc;
use std::sync::Arc;
use tracing::Instrument;
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
    let request_id = Uuid::new_v4();

    let request_span = tracing::info_span!(
    "Adding a new subscriber.",
    %request_id,
    subscriber_email = %form.email,
    subscriber_name= %form.name
    );
    let _request_span_guard = request_span.enter();

    let query_span = tracing::info_span!("Saving new subscriber details in the database");
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
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!("New subscriber details have been saved");
            StatusCode::OK
        }
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
