use crate::domain::SubscriberStatus;
use crate::{domain::NewSubscriber, email_client::EmailClient};
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
    skip(body, db, email),
    fields(
        subscriber_email = %body.email,
        subscriber_name = %body.name,
    ),
)]
pub async fn subscribe(
    body: web::Form<SubscribeFormBody>,
    db: web::Data<PgPool>,
    email: web::Data<EmailClient>,
) -> HttpResponse {
    let new_subscriber: NewSubscriber = match body.0.try_into() {
        Ok(sub) => sub,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    if let Err(e) = insert_subscriber(&db, &new_subscriber).await {
        tracing::error!("Failed to add new subscriber: {:?}", e);
        return HttpResponse::InternalServerError().finish();
    };

    if email
        .send_email(
            new_subscriber.email,
            "Welcome!",
            "Welcome to the newsletter",
            "Welcome to the newsletter",
        )
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    };

    HttpResponse::Ok().finish()
}

#[tracing::instrument(name = "Persisting subscriber to database", skip(db, body))]
pub async fn insert_subscriber(db: &PgPool, body: &NewSubscriber) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO subscriptions (id, email, name, subscribed_at, status) VALUES ($1, $2, $3, $4, $5)",
        Uuid::new_v4(),
        body.email.as_ref(),
        body.name.as_ref(),
        Utc::now(),
        SubscriberStatus::PendingConfirmation.to_string()
    )
    .execute(db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        println!("{:?}", e);
        e
    })?;

    Ok(())
}
