use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatabaseConnectionConfig {
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
    #[serde(default)]
    pub max_lifetime_secs: Option<u64>,
    #[serde(default)]
    pub idle_timeout_secs: Option<u64>,
}

impl Default for DatabaseConnectionConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            name: "postgres".to_string(),
            username: "postgres".to_string(),
            password: "".to_string(),
            max_connections: Some(10),
            min_connections: Some(1),
            acquire_timeout_secs: Some(30),
            max_lifetime_secs: Some(1800),
            idle_timeout_secs: Some(600),
        }
    }
}

impl DatabaseConnectionConfig {
    pub fn new(host: String, port: u16, name: String, username: String, password: String) -> Self {
        Self {
            host,
            port,
            name,
            username,
            password,
            ..Default::default()
        }
    }

    pub fn connection_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.name
        )
    }

    pub fn to_connect_options(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .database(&self.name)
            .username(&self.username)
            .password(&self.password)
    }

    pub fn to_pool_options(&self) -> PgPoolOptions {
        let mut pool_options = PgPoolOptions::new();

        if let Some(max) = self.max_connections {
            pool_options = pool_options.max_connections(max);
        }
        if let Some(min) = self.min_connections {
            pool_options = pool_options.min_connections(min);
        }
        if let Some(acquire_timeout) = self.acquire_timeout_secs {
            pool_options = pool_options.acquire_timeout(Duration::from_secs(acquire_timeout));
        }
        if let Some(max_lifetime) = self.max_lifetime_secs {
            pool_options = pool_options.max_lifetime(Duration::from_secs(max_lifetime));
        }
        if let Some(idle_timeout) = self.idle_timeout_secs {
            pool_options = pool_options.idle_timeout(Duration::from_secs(idle_timeout));
        }

        pool_options
    }
}
