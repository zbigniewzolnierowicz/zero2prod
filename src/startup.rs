use std::net::TcpListener;

use actix_web::dev::Server;
use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::{
    configuration::{DatabaseSettings, Settings},
    email_client::EmailClient,
    run,
};

pub struct Application {
    pub port: u16,
    pub server: Server,
}

impl Application {
    pub async fn build(config: Settings) -> Result<Self, std::io::Error> {
        let address = (config.application.host, config.application.port);
        let connection_pool = PgPoolOptions::new().connect_lazy_with(config.database.with_db());

        let sender_email = config.email.sender().expect("Bad sender email");
        let timeout = config.email.timeout();
        let email_client = EmailClient::new(
            config.email.base_url,
            sender_email,
            config.email.token,
            timeout,
        );

        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        let mut templates = tera::Tera::new("templates/**/*").expect("Could not load templates");
        templates.autoescape_on(vec![]);

        let server = run(
            listener,
            connection_pool,
            email_client,
            config.application.base_url,
            templates,
        )?;

        Ok(Self { port, server })
    }
}

pub fn get_connection_pool(config: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(config.with_db())
}
