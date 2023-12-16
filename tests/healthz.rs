use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use std::net::TcpListener;

use sqlx::{Connection, Executor, PgConnection, PgPool};
use zero2prod::{
    configuration::{DatabaseSettings, Settings},
    telemetry::{get_subscriber, init_subscriber},
};

struct TestApp {
    connection_string: String,
    database: PgPool,
}

static TRACING: Lazy<()> = Lazy::new(|| {
    let name = "test".to_string();
    let level = "debug".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(name, level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(name, level, std::io::sink);
        init_subscriber(subscriber);
    };
});

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("Failed to bind to a random port");
    let port = listener.local_addr().unwrap().port();

    let mut configuration = Settings::get().expect("Failed to read configuration");
    configuration.database.database_name = uuid::Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;

    let server = zero2prod::run(listener, connection_pool.clone()).expect("Could not bind address");
    tokio::spawn(server);

    TestApp {
        connection_string: format!("http://127.0.0.1:{}", port),
        database: connection_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect(&config.connection_string_no_db().expose_secret())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect(&config.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

#[tokio::test]
async fn test() {
    // GIVEN
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    // WHEN
    let result = client
        .get(format!("{}/healthz", app.connection_string))
        .send()
        .await
        .expect("Failed to send request");

    // THEN
    assert!(result.status().is_success());
    assert_eq!(result.content_length(), Some(0));
}

#[tokio::test]
async fn subscription_returns_200_on_correct_body() {
    // GIVEN
    let TestApp {
        connection_string,
        database,
    } = spawn_app().await;
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // WHEN
    let result = client
        .post(format!("{}/subscribe", connection_string))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Could not send request");

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&database)
        .await
        .expect("Failed to get subscriptions");

    // THEN
    assert_eq!(200, result.status());
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
}

#[tokio::test]
async fn subscription_returns_400_on_malformed_body() {
    // GIVEN
    let TestApp {
        connection_string, ..
    } = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = [
        ("name=lupin", "no email"),
        ("email=arsene@lup.in", "no name"),
        ("", "no name or email"),
    ];

    for (body, message) in test_cases {
        // WHEN
        let result = client
            .post(format!("{}/subscribe", connection_string))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Could not send request");

        // THEN
        assert_eq!(
            400,
            result.status(),
            "The API did not fail properly with Bad Request (400) when the body had {message}"
        );
    }
}
