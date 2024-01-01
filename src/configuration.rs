use secrecy::{ExposeSecret, Secret};
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::{postgres::PgConnectOptions, ConnectOptions};

use crate::domain::Email;

#[derive(strum::Display, Debug)]
pub enum Environment {
    #[strum(serialize = "dev")]
    Development,
    #[strum(serialize = "prod")]
    Production,
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "dev" => Ok(Self::Development),
            "prod" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a valid environment\nUse either `dev` or `prod`.",
                other
            )),
        }
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub email: EmailClientSettings,
}

#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    #[serde(default)]
    pub require_ssl: bool,
}

#[derive(serde::Deserialize, Clone )]
pub struct EmailClientSettings {
    pub base_url: String,
    pub sender_email: String,
    pub token: Secret<String>,
    pub timeout_miliseconds: u64,
}

impl EmailClientSettings {
    pub fn sender(&self) -> Result<Email, String> {
        Email::parse(self.sender_email.clone())
    }

    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_miliseconds)
    }
}

impl Settings {
    pub fn get() -> Result<Self, config::ConfigError> {
        let base_path = std::env::current_dir().expect("Could not get current directory");
        let config_path = base_path.join("config");

        let environment: Environment = std::env::var("APP_ENV")
            .unwrap_or_else(|_| "dev".into())
            .try_into()
            .expect("Failed to parse environment");

        let env_config = format!("{}.yaml", environment);

        let settings = config::Config::builder()
            .add_source(config::File::from(config_path.join("base.yaml")))
            .add_source(config::File::from(config_path.join(env_config)))
            .add_source(
                config::Environment::with_prefix("APP")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()?;
        settings.try_deserialize::<Self>()
    }
}

impl DatabaseSettings {
    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db()
            .database(&self.database_name)
            .log_statements(tracing_log::log::LevelFilter::Trace)
    }

    pub fn without_db(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .username(&self.username)
            .password(self.password.expose_secret())
    }
}
