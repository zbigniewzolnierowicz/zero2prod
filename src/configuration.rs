use secrecy::{ExposeSecret, Secret};

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

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
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
            .build()?;
        settings.try_deserialize::<Self>()
    }
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "{}/{}",
            self.connection_string_no_db().expose_secret(),
            self.database_name
        ))
    }
    pub fn connection_string_no_db(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        ))
    }
}
