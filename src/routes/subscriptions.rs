use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct SubscribeFormBody {
    name: String,
    email: String,
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
    match insert_subscriber(&db, &body).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            tracing::error!("Failed to add new subscriber: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[tracing::instrument(name = "Persisting subscriber to database", skip(db, body))]
pub async fn insert_subscriber(db: &PgPool, body: &SubscribeFormBody) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4) RETURNING id",
        Uuid::new_v4(),
        body.email, body.name,
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
