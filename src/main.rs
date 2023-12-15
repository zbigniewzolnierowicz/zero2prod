use actix_web::{web, App, HttpResponse, HttpServer, Responder};

async fn ping() -> impl Responder {
    HttpResponse::Ok()
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    HttpServer::new(|| App::new().route("/healthz", web::get().to(ping)))
        .bind(("127.0.0.1", 3000))?
        .run()
        .await
}
