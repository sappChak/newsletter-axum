use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Form};
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
    Extension(db): Extension<Arc<Database>>,
    Extension(ses_client): Extension<Arc<SESWorkflow>>,
    Extension(base_url): Extension<Arc<String>>,
    Form(form): Form<FormData>,
) -> impl IntoResponse {
    let new_subscriber = match form.try_into() {
        Ok(form) => form,
        Err(e) => {
            tracing::error!("Failed to parse subscriber details from form data: {:?}", e);
            return StatusCode::BAD_REQUEST;
        }
    };

    let mut transaction = match db.pool.begin().await {
        Ok(transaction) => transaction,
        Err(e) => {
            tracing::error!("Failed to create a new transaction {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    let subscriber_id = match insert_subscriber(&mut transaction, &new_subscriber).await {
        Ok(subscriber_id) => subscriber_id,
        Err(e) => {
            tracing::error!("Failed to insert new subscriber details: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    let subscription_token = generate_subscription_token();

    if store_token(&mut transaction, subscriber_id, &subscription_token)
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    if send_confirmation_email(
        ses_client,
        new_subscriber.email,
        &base_url,
        &subscription_token,
    )
    .await
    .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    if transaction.commit().await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(transaction, new_subscriber)
)]
pub async fn insert_subscriber(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    new_subscriber: &NewSubscriber,
) -> Result<uuid::Uuid, sqlx::Error> {
    let subscriber_id = uuid::Uuid::new_v4();
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
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(subscriber_id)
}

pub async fn send_confirmation_email(
    ses_client: Arc<SESWorkflow>,
    recipient_email: SubscriberEmail,
    base_url: &str,
    subscription_token: &str,
) -> Result<(), Box<dyn std::error::Error>> {
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
        .send_email(recipient_email, subject, &text_content, &html_content)
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
) -> Result<(), sqlx::Error> {
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
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
