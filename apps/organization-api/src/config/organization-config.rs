use config::{Config, ConfigError};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct OrganizationAppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
}

impl OrganizationAppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let settings = Config::builder()
            .add_source(config::File::with_name("apps/organization-api/cfg/default"))
            .add_source(
                config::Environment::with_prefix("APP")
                    .separator("__"),
            )
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
        assert_eq!(*shared_config, *shared_clone);
    }
}