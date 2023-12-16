use std::net::TcpListener;

use sqlx::PgPool;
use zero2prod::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let config = zero2prod::configuration::Settings::get().expect("Failed to read configuration.");
    let address = ("127.0.0.1", config.port);
    let connection_pool = PgPool::connect(&config.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    run(TcpListener::bind(address).expect("Could not bind to address"), connection_pool)?.await
}
