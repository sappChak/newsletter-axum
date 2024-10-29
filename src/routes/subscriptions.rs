use crate::db::database::Database;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Form};
use sqlx::types::chrono::Utc;
use std::sync::Arc;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(db, form),
    fields(
      request_id = %uuid::Uuid::new_v4(),
      subscriber_email = %form.email,
      subscriber_name= %form.name
    )
)]
pub async fn subscribe(
    Extension(db): Extension<Arc<Database>>,
    Form(form): Form<FormData>,
) -> impl IntoResponse {
    match insert_subscriber(db, form).await {
        Ok(_) => {
            tracing::info!("New subscriber details have been saved");
            StatusCode::OK
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[tracing::instrument(name = "Saving new subscriber details in the database", skip(db, form))]
pub async fn insert_subscriber(db: Arc<Database>, form: FormData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
          INSERT INTO subscriptions (id, email, name, subscribed_at)
          VALUES ($1, $2, $3, $4)
        "#,
        uuid::Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(&db.pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
