use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

#[derive(serde::Deserialize)]
pub struct SubscribeFormBody {
    name: String,
    email: String,
}

pub async fn subscribe(body: web::Form<SubscribeFormBody>, db: web::Data<PgPool>) -> HttpResponse {
    let request_id = Uuid::new_v4();
    log::info!("REQUEST {request_id} | Adding a new subscriber: {} <{}>", body.name, body.email);
    match sqlx::query!(
        "INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4) RETURNING id",
        Uuid::new_v4(),
        body.email,
        body.name,
        Utc::now()
    )
    .fetch_one(db.get_ref())
    .await {
        Ok(subscriber) => { 
            log::info!("REQUEST {request_id} | New subscriber saved. ID: {}", subscriber.id);
            HttpResponse::Ok().finish()
        },
        Err(e) => {
            log::error!("REQUEST {request_id} | Failed to add new subscriber: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
