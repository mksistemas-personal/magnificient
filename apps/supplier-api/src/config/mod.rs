use config::{Config, ConfigError};
use database::DatabaseConnectionConfig;
use messaging::RabbitMqConfig;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SupplierAppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConnectionConfig,
    pub rabbitmq: RabbitMqConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
}

impl SupplierAppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(config::File::with_name("apps/supplier-api/cfg/default").required(false))
            .add_source(config::Environment::with_prefix("APP").separator("__"))
            .build()?
            .try_deserialize()
    }
}
