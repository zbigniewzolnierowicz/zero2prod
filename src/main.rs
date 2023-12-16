use std::net::TcpListener;
use env_logger::Env;

use sqlx::PgPool;
use zero2prod::{run, configuration::Settings};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let config = Settings::get().expect("Failed to read configuration.");
    let address = ("127.0.0.1", config.port);
    let connection_pool = PgPool::connect(&config.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    run(TcpListener::bind(address).expect("Could not bind to address"), connection_pool)?.await
}
