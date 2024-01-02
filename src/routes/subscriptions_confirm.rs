use crate::domain::SubscriberStatus;
use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ConfirmParameters {
    token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(db, params))]
pub async fn confirm(db: web::Data<PgPool>, params: web::Query<ConfirmParameters>) -> HttpResponse {
    let subscriber_id = match get_subscriber_id_from_token(&db, &params.token).await {
        Ok(id) => id,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    match subscriber_id {
        // Token does not exist
        None => HttpResponse::Unauthorized().finish(),
        Some(subscriber_id) => {
            match check_if_subscriber_confirmed(&db, &subscriber_id).await {
                Ok(true) => return HttpResponse::BadRequest().finish(),
                Err(_) => return HttpResponse::InternalServerError().finish(),
                _ => {}
            };

            if confirm_subscriber(&db, &subscriber_id).await.is_err() {
                return HttpResponse::InternalServerError().finish();
            }
            HttpResponse::Ok().finish()
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
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

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
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
