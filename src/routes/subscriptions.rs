use crate::domain::NewSubscriber;
use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct SubscribeFormBody {
    pub name: String,
    pub email: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(body, db),
    fields(
        subscriber_email = %body.email,
        subscriber_name = %body.name,
    ),
)]

pub async fn subscribe(body: web::Form<SubscribeFormBody>, db: web::Data<PgPool>) -> HttpResponse {
    let new_subscriber: NewSubscriber = match body.0.try_into() {
        Ok(sub) => sub,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match insert_subscriber(&db, &new_subscriber).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            tracing::error!("Failed to add new subscriber: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[tracing::instrument(name = "Persisting subscriber to database", skip(db, body))]
pub async fn insert_subscriber(db: &PgPool, body: &NewSubscriber) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4) RETURNING id",
        Uuid::new_v4(),
        body.email.as_ref(),
        body.name.as_ref(),
        Utc::now()
    )
    .fetch_one(db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
