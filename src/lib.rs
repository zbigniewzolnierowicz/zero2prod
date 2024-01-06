use crate::configuration::ApplicationBaseUrl;
use crate::email_client::EmailClient;
use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgPool;
use tera::Tera;
use tracing_actix_web::TracingLogger;

pub mod configuration;
pub mod domain;
pub mod email_client;
pub mod routes;
pub mod startup;
pub mod telemetry;
pub mod utils;

pub fn run(
    listener: std::net::TcpListener,
    database: PgPool,
    email_client: EmailClient,
    base_url: ApplicationBaseUrl,
    templates: Tera,
) -> Result<Server, std::io::Error> {
    let database = web::Data::new(database);
    let email_client = web::Data::new(email_client);
    let base_url = web::Data::new(base_url);
    let tera = web::Data::new(templates);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/healthz", web::get().to(routes::ping))
            .route("/subscribe", web::post().to(routes::subscribe))
            .route("/subscribe/confirm", web::get().to(routes::confirm))
            .app_data(database.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
            .app_data(tera.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
