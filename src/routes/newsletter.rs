use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;

use super::error_chain_fmt;
use crate::{database::db::Database, domain::SubscriberEmail, ses_workflow::SESWorkflow};

#[derive(serde::Deserialize)]
pub struct NewsletterPayload {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    text: String,
    html: String,
}

pub async fn publish_newsletter(
    State(ses_client): State<Arc<SESWorkflow>>,
    State(db): State<Arc<Database>>,
    Json(payload): Json<NewsletterPayload>,
) -> Result<Response, PublishError> {
    let subscribers = get_confirmed_subscribers(&db.pool).await?;
    for subscriber in subscribers {
        match subscriber {
            Ok(sub) => {
                ses_client
                    .send_email(
                        &sub.email,
                        &payload.title,
                        &payload.content.text,
                        &payload.content.html,
                    )
                    .await
                    // Lazy load context, avoid paying for the error path when fallible operations succeeds.
                    .with_context(|| {
                        format!("Failed to send newsletter issue to {}", &sub.email)
                    })?;
            }
            Err(error) => {
                tracing::warn!(
                // We record the error chain as a structured field
                // on the log record.
                error.cause_chain = ?error,
                // Using `\` to split a long string literal over
                // two lines, without creating a `\n` character.
                "Skipping a confirmed subscriber. \
                Their stored contact details are invalid",
                );
            }
        }
    }

    let response_body =
        Json(serde_json::json!({ "message": "Newsletter was published successfully." }));
    Ok((StatusCode::OK, response_body).into_response())
}

#[derive(thiserror::Error)]
pub enum PublishError {
    // Delegate source and Display to anyhow::Error
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for PublishError {
    fn into_response(self) -> Response {
        #[derive(serde::Serialize)]
        struct Error {
            message: String,
        }

        let (status, message) = match self {
            Self::UnexpectedError(e) => {
                tracing::error!("Got an unexpected one: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
        };

        (status, Json(Error { message })).into_response()
    }
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[tracing::instrument(name = "Get confirmed subscribers")]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    struct Row {
        email: String,
    }
    // Map to the Row type and minimise amount of data to be fetched
    let rows = sqlx::query_as!(
        Row,
        r#"SELECT email FROM subscriptions WHERE status='confirmed'"#,
    )
    .fetch_all(pool)
    .await?;

    let confirmed_subscribers = rows
        .into_iter()
        .map(|r| match SubscriberEmail::parse(r.email) {
            Ok(email) => Ok(ConfirmedSubscriber { email }),
            Err(error) => Err(anyhow::anyhow!(error)),
        })
        .collect();

    Ok(confirmed_subscribers)
}
