use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use zero2prod::{configuration::Settings, run};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let config = Settings::get().expect("Failed to read configuration.");
    let address = (config.application.host, config.application.port);
    let connection_pool = PgPool::connect_lazy(&config.database.connection_string().expose_secret())
        .expect("Failed to create Postgres connection pool.");
    let listener = TcpListener::bind(address)?;

    run(listener, connection_pool)?.await
}
