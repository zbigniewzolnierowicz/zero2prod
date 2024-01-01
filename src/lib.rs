use crate::email_client::EmailClient;
use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

pub mod configuration;
pub mod domain;
pub mod email_client;
pub mod routes;
pub mod startup;
pub mod telemetry;

pub fn run(
    listener: std::net::TcpListener,
    database: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    let database = web::Data::new(database);
    let email_client = web::Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/healthz", web::get().to(routes::ping))
            .route("/subscribe", web::post().to(routes::subscribe))
            .app_data(database.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
