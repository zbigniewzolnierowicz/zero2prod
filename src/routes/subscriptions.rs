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
    match sqlx::query!(
        "INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4)",
        Uuid::new_v4(),
        body.email,
        body.name,
        Utc::now()
    )
    .execute(db.get_ref())
    .await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("Could not add subscription: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
