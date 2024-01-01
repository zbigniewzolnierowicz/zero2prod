use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use zero2prod::email_client::EmailClient;
use zero2prod::telemetry;
use zero2prod::{configuration::Settings, run};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = telemetry::get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let config = Settings::get().expect("Failed to read configuration.");
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

    run(listener, connection_pool, email_client)?.await
}
