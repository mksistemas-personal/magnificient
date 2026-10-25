use config::{Config, ConfigError};
use database::DatabaseConnectionConfig;
use messaging::RabbitMqConfig;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct OrganizationAppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub rabbitmq: RabbitMqConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub max_connections: Option<u32>,
    #[serde(default)]
    pub min_connections: Option<u32>,
    #[serde(default)]
    pub acquire_timeout_secs: Option<u64>,
}

impl DatabaseConfig {
    pub fn connection_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.name
        )
    }

    pub fn to_connection_config(&self) -> DatabaseConnectionConfig {
        DatabaseConnectionConfig {
            host: self.host.clone(),
            port: self.port,
            name: self.name.clone(),
            username: self.username.clone(),
            password: self.password.clone(),
            max_connections: self.max_connections,
            min_connections: self.min_connections,
            acquire_timeout_secs: self.acquire_timeout_secs,
            ..Default::default()
        }
    }
}

impl From<&DatabaseConfig> for DatabaseConnectionConfig {
    fn from(config: &DatabaseConfig) -> Self {
        config.to_connection_config()
    }
}

impl From<DatabaseConfig> for DatabaseConnectionConfig {
    fn from(config: DatabaseConfig) -> Self {
        config.to_connection_config()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
}

impl OrganizationAppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let settings = Config::builder()
            .add_source(config::File::with_name("apps/organization-api/cfg/default"))
            .add_source(config::Environment::with_prefix("APP").separator("__"))
            .build()?;
        settings.try_deserialize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_organization_app_config_arc_sharing() {
        let config = OrganizationAppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
            },
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                name: "magnificient".to_string(),
                username: "admin".to_string(),
                password: "secret".to_string(),
                max_connections: None,
                min_connections: None,
                acquire_timeout_secs: None,
            },
            rabbitmq: RabbitMqConfig {
                rabbitmq_url: "amqp://rabbitmq:rabbitmq@localhost:5672/%2f".to_string(),
                rabbitmq_prefetch_count: 10,
                rabbitmq_reconnect_max_attempts: 0,
                rabbitmq_reconnect_initial_ms: 1000,
                rabbitmq_reconnect_max_ms: 30000,
                rabbitmq_retry_max_attempts: 3,
                rabbitmq_retry_initial_ms: 1000,
                rabbitmq_retry_max_ms: 10000,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
            },
        };

        let shared_config = Arc::new(config.clone());
        let shared_clone = Arc::clone(&shared_config);

        assert_eq!(shared_config.server.host, "127.0.0.1");
        assert_eq!(shared_config.server.port, 8080);
        assert_eq!(shared_clone.database.name, "magnificient");
        assert_eq!(
            shared_clone.database.connection_url(),
            "postgres://admin:secret@localhost:5432/magnificient"
        );
        let conn_config = shared_clone.database.to_connection_config();
        assert_eq!(conn_config.name, "magnificient");
        assert_eq!(*shared_config, *shared_clone);
    }
}
