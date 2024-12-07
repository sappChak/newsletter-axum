use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Form};
use sqlx::types::chrono::Utc;

use std::sync::Arc;

use crate::database::db::Database;
use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use crate::ses_workflow::SESWorkflow;

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

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(db, ses_client, form),
    fields(
      subscriber_email = %form.email,
      subscriber_name= %form.name
    )
)]
pub async fn subscribe(
    Extension(db): Extension<Arc<Database>>,
    Extension(ses_client): Extension<Arc<SESWorkflow>>,
    Form(form): Form<FormData>,
) -> impl IntoResponse {
    let new_subscriber = match form.try_into() {
        Ok(form) => form,
        Err(_) => return StatusCode::BAD_REQUEST,
    };
    if insert_subscriber(db, &new_subscriber).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    if send_confirmation_email(ses_client, new_subscriber.email)
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(db, new_subscriber)
)]
pub async fn insert_subscriber(
    db: Arc<Database>,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
          INSERT INTO subscriptions (id, email, name, subscribed_at, status)
          VALUES ($1, $2, $3, $4, 'confirmed')
        "#,
        uuid::Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
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

pub async fn send_confirmation_email(
    ses_client: Arc<SESWorkflow>,
    recipient_email: SubscriberEmail,
) -> Result<(), Box<dyn std::error::Error>> {
    let confirmation_link = "https://something.com/confirm";
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
        .send_email(recipient_email, "Welcome!", &text_content, &html_content)
        .await?;

    Ok(())
}
