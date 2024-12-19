use std::sync::Arc;

use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Extension;
use sqlx::PgPool;

use crate::database::db::Database;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters, db))]
pub async fn confirm(
    Extension(db): Extension<Arc<Database>>,
    Query(parameters): Query<Parameters>,
) -> impl IntoResponse {
    let subscriber_id =
        match get_subscriber_id_from_token(&db.pool, &parameters.subscription_token).await {
            Ok(subscriber_id) => subscriber_id,
            Err(_) => return StatusCode::BAD_REQUEST,
        };

    match subscriber_id {
        Some(subscriber_id) => {
            if confirm_subscriber(&db.pool, subscriber_id).await.is_ok() {
                StatusCode::OK
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
        None => StatusCode::NOT_FOUND,
    };

    StatusCode::OK
}

#[tracing::instrument(name = "Get subscriber ID from token", skip(pool, subscription_token))]
pub async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<uuid::Uuid>, sqlx::Error> {
    let subscriber_id = sqlx::query!(
        r#"
          SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1
        "#,
        &subscription_token
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(subscriber_id.map(|id| id.subscriber_id))
}

#[tracing::instrument(name = "Update subscription status", skip(pool))]
pub async fn confirm_subscriber(
    pool: &PgPool,
    subscriber_id: uuid::Uuid,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query!(
        r#"
      UPDATE subscriptions SET status = 'confirmed' WHERE id = $1
    "#,
        subscriber_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    });

    Ok(())
}
