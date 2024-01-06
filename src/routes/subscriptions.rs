use crate::configuration::ApplicationBaseUrl;
use crate::domain::{Email, SubscriberStatus};
use crate::{domain::NewSubscriber, email_client::EmailClient};
use actix_web::{web, HttpResponse, ResponseError};
use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, PgPool, Postgres, Row, Transaction};
use tera::{Context, Tera};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct SubscribeFormBody {
    pub name: String,
    pub email: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(body, db, email, base_url, template),
    fields(
        subscriber_email = %body.email,
        subscriber_name = %body.name,
    ),
)]
pub async fn subscribe(
    body: web::Form<SubscribeFormBody>,
    db: web::Data<PgPool>,
    email: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
    template: web::Data<Tera>,
) -> Result<HttpResponse, SubscribeError> {
    let new_subscriber: NewSubscriber = body.0.try_into()?;
    let mut tx = db.begin().await.map_err(SubscribeError::PoolError)?;
    let subscriber_id = insert_subscriber(&mut tx, &new_subscriber)
        .await
        .map_err(SubscribeError::InsertSubscriberError)?;
    let subscription_token = store_token(&mut tx, &subscriber_id).await?;

    tx.commit()
        .await
        .map_err(SubscribeError::TransactionCommitError)?;

    send_confirmation_email(
        &email,
        &template,
        &new_subscriber,
        &base_url.0,
        &subscription_token,
    )
    .await?;

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(name = "Persisting subscription token in database", skip(tx))]
pub async fn store_token(
    tx: &mut Transaction<'_, Postgres>,
    subscriber_id: &Uuid,
) -> Result<String, StoreTokenError> {
    let token_already_exists_query = sqlx::query!(
        "SELECT subscription_token FROM subscription_tokens WHERE subscriber_id = $1",
        subscriber_id
    );
    if let Some(record) = tx
        .fetch_optional(token_already_exists_query)
        .await
        .map_err(StoreTokenError)?
    {
        return Ok(record.get("subscription_token"));
    };

    let token = generate_subscription_token();

    let query = sqlx::query!(
        "INSERT INTO subscription_tokens (subscription_token, subscriber_id) VALUES ($1, $2)",
        token,
        subscriber_id
    );

    tx.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        StoreTokenError(e)
    })?;

    Ok(token)
}

pub fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[tracing::instrument(name = "Checking if subscriber already exists", skip(tx, email))]
pub async fn does_subscriber_exist(
    tx: &mut Transaction<'_, Postgres>,
    email: &Email,
) -> Result<Option<Uuid>, sqlx::Error> {
    let user_already_exists_query = sqlx::query!(
        "SELECT id FROM subscriptions WHERE email = $1",
        email.as_ref()
    );
    if let Some(record) = tx.fetch_optional(user_already_exists_query).await? {
        Ok(Some(record.get("id")))
    } else {
        Ok(None)
    }
}

#[tracing::instrument(name = "Persisting subscriber to database", skip(tx, body))]
pub async fn insert_subscriber(
    tx: &mut Transaction<'_, Postgres>,
    body: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let new_subscriber_id = match does_subscriber_exist(tx, &body.email).await? {
        Some(id) => return Ok(id),
        None => Uuid::new_v4(),
    };
    let query = sqlx::query!(
        "INSERT INTO subscriptions (id, email, name, subscribed_at, status) VALUES ($1, $2, $3, $4, $5)",
        new_subscriber_id,
        body.email.as_ref(),
        body.name.as_ref(),
        Utc::now(),
        SubscriberStatus::PendingConfirmation.to_string()
    );

    tx.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(new_subscriber_id)
}

#[derive(Serialize)]
struct ConfirmationEmailContext<'a> {
    name: &'a str,
    link: String,
}

#[tracing::instrument(
    name = "Sending a confirmation email to user",
    skip(email, new_subscriber, base_url, template)
)]
pub async fn send_confirmation_email(
    email: &EmailClient,
    template: &Tera,
    new_subscriber: &NewSubscriber,
    base_url: &str,
    token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!("{base_url}/subscribe/confirm?token={token}");

    let context = ConfirmationEmailContext {
        name: new_subscriber.name.as_ref(),
        link: confirmation_link,
    };

    let html_body = template
        .render(
            "confirm-email.html",
            &Context::from_serialize(&context).unwrap(),
        )
        .unwrap();
    let text_body = template
        .render(
            "confirm-email.txt",
            &Context::from_serialize(&context).unwrap(),
        )
        .unwrap();

    email
        .send_email(
            new_subscriber.email.clone(),
            "Welcome!",
            &html_body,
            &text_body,
        )
        .await
}

fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
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

impl std::fmt::Debug for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl std::error::Error for StoreTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    ValidationError(String),

    #[error("Failed to store confirmation token for new subscriber.")]
    StoreTokenError(#[from] StoreTokenError),

    #[error("Failed to send confirmation email")]
    SendEmailError(#[from] reqwest::Error),

    #[error("Failed to acquire Postgres connection from pool")]
    PoolError(#[source] sqlx::Error),

    #[error("Failed to insert new subscriber in database")]
    InsertSubscriberError(#[source] sqlx::Error),

    #[error("Failed to commit SQL transaction to store a new subscriber")]
    TransactionCommitError(#[source] sqlx::Error),
}

impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl From<String> for SubscribeError {
    fn from(value: String) -> Self {
        Self::ValidationError(value)
    }
}

impl ResponseError for SubscribeError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::ValidationError(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
