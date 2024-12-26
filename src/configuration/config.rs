use config::builder::DefaultState;
use secrecy::{ExposeSecret, SecretString};
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};

use super::environment::Environment;

#[derive(serde::Deserialize)]
pub struct Configuration {
    pub database: DatabaseConfiguration,
    pub application: ApplicationConfiguration,
    pub aws: AwsConfiguration,
}

#[derive(serde::Deserialize)]
pub struct DatabaseConfiguration {
    pub username: String,
    pub password: SecretString,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

#[derive(serde::Deserialize)]
pub struct ApplicationConfiguration {
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub base_url: String,
    pub logger_name: String,
    pub default_env_filter: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct AwsConfiguration {
    pub region: String,
    pub verified_email: String,
    pub access_key_id: String,
    pub secret_access_key: String,
}

impl DatabaseConfiguration {
    pub fn without_db(&self) -> PgConnectOptions {
        // Try an encrypted connection, fallback to unencrypted if it fails
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.database_name)
    }
}

pub fn get_configuration() -> Result<Configuration, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to get current directory.");
    let config_directory = base_path.join("configuration");

    let builder = config::ConfigBuilder::<DefaultState>::default()
        .add_source(config::File::from(config_directory.join("base")).required(true));

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to read APP_ENVIRONMENT");

    let config = builder
        .add_source(config::File::from(config_directory.join(environment.as_str())).required(true))
        .add_source(config::Environment::with_prefix("app").separator("__"))
        .build()?;

    config.try_deserialize::<Configuration>()
}
