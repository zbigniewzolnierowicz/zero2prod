use fake::{
    faker::{internet::en::SafeEmail, name::en::FirstName},
    Fake,
};
use once_cell::sync::Lazy;

use sqlx::{postgres::PgPoolOptions, Executor, PgPool};
use wiremock::MockServer;
use zero2prod::{
    configuration::{DatabaseSettings, Settings},
    startup::get_connection_pool,
    telemetry::{get_subscriber, init_subscriber},
};

pub struct TestApp {
    pub connection_string: String,
    pub port: u16,
    pub database: PgPool,
    pub email_server: MockServer,
}

pub fn name() -> String {
    FirstName().fake()
}

pub fn email() -> String {
    SafeEmail().fake()
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

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let email_server = MockServer::start().await;

    let config = {
        let mut config = Settings::get().expect("Failed to read configuration");
        config.database.database_name = uuid::Uuid::new_v4().to_string();
        config.application.port = 0;
        config.email.base_url = email_server.uri();

        config
    };

    configure_database(&config.database).await;

    let app = zero2prod::startup::Application::build(config.clone())
        .await
        .expect("Failed to build app.");

    let port = app.port;

    tokio::spawn(app.server);

    TestApp {
        database: get_connection_pool(&config.database),
        connection_string: format!("http://127.0.0.1:{}", app.port),
        email_server,
        port,
    }
}

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscribe", self.connection_string))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Could not send request")
    }

    pub async fn healthcheck(&self) -> reqwest::Response {
        reqwest::Client::new()
            .get(format!("{}/healthz", self.connection_string))
            .send()
            .await
            .expect("Failed to send request")
    }

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();
        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();

            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_string();

            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");

            confirmation_link.set_port(Some(self.port)).unwrap();
            confirmation_link
        };

        let html = get_link(body["HtmlBody"].as_str().unwrap());
        let plain_text = get_link(body["TextBody"].as_str().unwrap());

        ConfirmationLinks { html, plain_text }
    }

    pub async fn post_newsletters(&self, body: serde_json::Value) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/newsletters", &self.connection_string))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub async fn configure_database(config: &DatabaseSettings) {
    // Create database
    let connection = PgPoolOptions::new()
        .connect_with(config.without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPoolOptions::new()
        .connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
}
