use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgPool;


pub mod configuration;
mod startup;
mod routes;

pub fn run(listener: std::net::TcpListener, database: PgPool) -> Result<Server, std::io::Error> {
    let database = web::Data::new(database);
    let server = HttpServer::new(move || {
        App::new()
            .route("/healthz", web::get().to(routes::ping))
            .route("/subscribe", web::post().to(routes::subscribe))
            .app_data(database.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}

