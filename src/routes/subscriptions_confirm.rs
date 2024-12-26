use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use sqlx::PgPool;

use crate::database::db::Database;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[derive(thiserror::Error, Debug)]
pub enum ConfirmationError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("There is no subscriber associated with the provided token.")]
    UnknownToken,
}

impl IntoResponse for ConfirmationError {
    fn into_response(self) -> Response {
        #[derive(serde::Serialize)]
        struct Error {
            message: String,
        }

        let (status, message) = match self {
            ConfirmationError::UnexpectedError(error) => {
                tracing::error!("Got an unexpected one: {}", error);
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
            }
            ConfirmationError::UnknownToken => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Achtung, unknown token error".to_string(),
            ),
        };

        (status, Json(Error { message })).into_response()
    }
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters, db))]
pub async fn confirm(
    Extension(db): Extension<Arc<Database>>,
    Query(parameters): Query<Parameters>,
) -> Result<Response, ConfirmationError> {
    let subscriber_id = get_subscriber_id_from_token(&db.pool, &parameters.subscription_token)
        .await
        .context("Failed to get subscriber id.")?
        .ok_or(ConfirmationError::UnknownToken)?;

    confirm_subscriber(&db.pool, subscriber_id)
        .await
        .context("Failed to confirm subscriber.")?;

    Ok(StatusCode::OK.into_response())
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
    .await?;

    Ok(subscriber_id.map(|id| id.subscriber_id))
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(pool))]
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
    .await?;

    Ok(())
}
