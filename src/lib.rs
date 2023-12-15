use actix_web::{dev::Server, web, App, HttpResponse, HttpServer};

async fn ping() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[derive(serde::Deserialize)]
struct SubscribeFormBody {
    name: String,
    email: String,
}

async fn subscribe(body: web::Form<SubscribeFormBody>) -> HttpResponse {
    HttpResponse::Ok().body(format!("Hello, {} <{}>!", body.name, body.email))
}

pub fn run(listener: std::net::TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/healthz", web::get().to(ping))
            .route("/subscribe", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();

    Ok(server)
}

#[cfg(test)]
mod tests {
    use crate::ping;

    #[tokio::test]
    async fn ping_works() {
        // GIVEN, WHEN
        let result = ping().await;

        // THEN
        assert!(result.status().is_success());
    }
}
