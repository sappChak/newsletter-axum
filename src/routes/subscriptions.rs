use std::sync::Arc;

use anyhow::{Context, Result};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Form, Json,
};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sqlx::types::chrono::Utc;
use uuid::Uuid;

use crate::{
    database::db::Database,
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    ses_workflow::SESWorkflow,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self { email, name })
    }
}

#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for SubscribeError {
    fn into_response(self) -> Response {
        #[derive(serde::Serialize)]
        struct Error {
            message: String,
        }

        let (status, message) = match self {
            Self::ValidationError(e) => {
                tracing::error!("Validation error occured: {:?}", e);
                (StatusCode::BAD_REQUEST, e.to_string())
            }
            Self::UnexpectedError(e) => {
                tracing::error!("Got an unexpected one: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
        };

        (status, Json(Error { message })).into_response()
    }
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(db, ses_client, form, base_url),
    fields(
      subscriber_email = %form.email,
      subscriber_name= %form.name
    )
)]
pub async fn subscribe(
    State(db): State<Arc<Database>>,
    State(ses_client): State<Arc<SESWorkflow>>,
    Extension(base_url): Extension<Arc<String>>,
    Form(form): Form<FormData>,
) -> Result<Response, SubscribeError> {
    let new_subscriber = form.try_into().map_err(SubscribeError::ValidationError)?;

    let mut transaction = db
        .pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool.")?;

    let subscriber_id = insert_subscriber(&mut transaction, &new_subscriber)
        .await
        .context("Failed to insert new subscriber in the database.")?;

    let subscription_token = generate_subscription_token();
    store_token(&mut transaction, subscriber_id, &subscription_token)
        .await
        .context("Failed to store the confirmation token for a new subscriber.")?;

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new subscriber.")?;

    send_confirmation_email(
        ses_client,
        new_subscriber.email,
        &base_url,
        &subscription_token,
    )
    .await
    .context("Failed to send a confirmation email.")?;

    let response_body = Json(serde_json::json!({ "message": "Check your e-mail box, please" }));
    Ok((StatusCode::OK, response_body).into_response())
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(transaction, new_subscriber)
)]
pub async fn insert_subscriber(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    new_subscriber: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
          INSERT INTO subscriptions (id, email, name, subscribed_at, status)
          VALUES ($1, $2, $3, $4, 'pending_confirmation')
        "#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(&mut **transaction)
    .await?;

    Ok(subscriber_id)
}

pub async fn send_confirmation_email(
    ses_client: Arc<SESWorkflow>,
    recipient_email: SubscriberEmail,
    base_url: &str,
    subscription_token: &str,
) -> Result<()> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, subscription_token
    );

    let subject = "Welcome to our newsletter!";
    let text_content = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    let html_content = format!(
        r#"
        <html>
            <body>
                <h1>Welcome to our newsletter!</h1>
                <p>Visit <a href="{}">{}</a> to confirm your subscription.</p>
            </body>
        </html>
        "#,
        confirmation_link, confirmation_link
    );

    ses_client
        .send_email(&recipient_email, subject, &text_content, &html_content)
        .await?;

    Ok(())
}

#[tracing::instrument(
    name = "Store subscription token in the database",
    skip(transaction, subscription_token)
)]
pub async fn store_token(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), StoreTokenError> {
    sqlx::query!(
        r#"
          INSERT INTO subscription_tokens (subscription_token, subscriber_id)
          VALUES ($1, $2)
        "#,
        subscription_token,
        subscriber_id
    )
    .execute(&mut **transaction)
    .await
    .map_err(StoreTokenError)?;

    Ok(())
}

pub struct StoreTokenError(sqlx::Error);

impl std::fmt::Display for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database error was encountered while \
            trying to store a subscription token."
        )
    }
}

impl std::error::Error for StoreTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl std::fmt::Debug for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}
