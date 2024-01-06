use crate::{domain::SubscriberStatus, utils::error_chain_fmt};
use actix_web::{web, HttpResponse, ResponseError};
use anyhow::Context;
use reqwest::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ConfirmParameters {
    token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(db, params))]
pub async fn confirm(
    db: web::Data<PgPool>,
    params: web::Query<ConfirmParameters>,
) -> Result<HttpResponse, ConfirmSubscriptionError> {
    let subscriber_id = get_subscriber_id_from_token(&db, &params.token)
        .await
        .context("Could not get subscriber ID from token")?;

    match subscriber_id {
        // Token does not exist
        None => Err(ConfirmSubscriptionError::SubscriberDoesNotExist),
        Some(subscriber_id) => {
            if check_if_subscriber_confirmed(&db, &subscriber_id)
                .await
                .context("Could not check if subscriber was confirmed")?
            {
                return Err(ConfirmSubscriptionError::SubscriberAlreadyConfirmedError);
            };

            confirm_subscriber(&db, &subscriber_id)
                .await
                .context("Could not confirm subscriber")?;
            Ok(HttpResponse::Ok().finish())
        }
    }
}

#[tracing::instrument(name = "Check if subscriber is already confirmed", skip(db))]
pub async fn check_if_subscriber_confirmed(
    db: &PgPool,
    user_id: &Uuid,
) -> Result<bool, sqlx::Error> {
    let subscriber_status = sqlx::query!(
        "SELECT status FROM subscriptions WHERE id = $1 AND status = $2",
        user_id,
        SubscriberStatus::Ok.to_string()
    )
    .fetch_optional(db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(subscriber_status.is_some())
}

#[tracing::instrument(name = "Getting subscriber ID from token", skip(db, token))]
pub async fn get_subscriber_id_from_token(
    db: &PgPool,
    token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT subscriber_id FROM subscription_tokens \
                              WHERE subscription_token = $1",
        token
    )
    .fetch_optional(db)
    .await?;

    Ok(result.map(|r| r.subscriber_id))
}

#[tracing::instrument(name = "Confirming user's subscription", skip(db))]
pub async fn confirm_subscriber(db: &PgPool, user_id: &Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE subscriptions SET status = $2 WHERE id = $1",
        user_id,
        SubscriberStatus::Ok.to_string()
    )
    .execute(db)
    .await?;

    Ok(())
}

#[derive(thiserror::Error)]
pub enum ConfirmSubscriptionError {
    #[error("Subscriber was already confirmed")]
    SubscriberAlreadyConfirmedError,

    #[error("Subscriber does not exist.")]
    SubscriberDoesNotExist,

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for ConfirmSubscriptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for ConfirmSubscriptionError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::SubscriberAlreadyConfirmedError => StatusCode::BAD_REQUEST,
            Self::SubscriberDoesNotExist => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
