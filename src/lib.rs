use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

pub mod configuration;
pub mod domain;
pub mod routes;
pub mod startup;
pub mod telemetry;

pub fn run(listener: std::net::TcpListener, database: PgPool) -> Result<Server, std::io::Error> {
    let database = web::Data::new(database);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/healthz", web::get().to(routes::ping))
            .route("/subscribe", web::post().to(routes::subscribe))
            .app_data(database.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
