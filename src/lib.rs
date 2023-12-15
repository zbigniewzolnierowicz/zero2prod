
use actix_web::{dev::Server, web, App, HttpResponse, HttpServer};

async fn ping() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn run(listener: std::net::TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| App::new().route("/healthz", web::get().to(ping)))
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
