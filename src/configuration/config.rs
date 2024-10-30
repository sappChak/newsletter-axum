use config::builder::DefaultState;
use secrecy::{ExposeSecret, SecretString};

use super::environment::Environment;

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretString,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    pub host: String,
    pub port: u16,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> SecretString {
        SecretString::new(
            format!(
                "postgres://{}:{}@{}:{}/{}",
                self.username,
                self.password.expose_secret(),
                self.host,
                self.port,
                self.database_name
            )
            .into(),
        )
    }

    pub fn connection_string_without_db(&self) -> SecretString {
        SecretString::new(
            format!(
                "postgres://{}:{}@{}:{}",
                self.username,
                self.password.expose_secret(),
                self.host,
                self.port
            )
            .into(),
        )
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to get current directory.");
    let config_directory: std::path::PathBuf = base_path.join("configuration");

    let builder = config::ConfigBuilder::<DefaultState>::default()
        .add_source(config::File::from(config_directory.join("base")).required(true));

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to read APP_ENVIRONMENT");

    // TODO: create config file if one doesn't exist

    let config = builder
        .add_source(config::File::from(config_directory.join(environment.as_str())).required(true))
        .build()?;

    config.try_deserialize::<Settings>()
}
