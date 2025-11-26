use config::Environment;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Default)]
#[allow(dead_code)]
pub struct ApplicationConfig {
    pub config: ConfigInfo,
    pub port: Option<u16>,
    pub database: Database,
    pub event_provider_client: EventProviderClient,
    pub api: Api,
}

#[derive(Clone, Debug, Deserialize, Default)]
#[allow(dead_code)]
pub struct ConfigInfo {
    pub env_prefix: String,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct Database {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct EventProviderClient {
    pub url: String,
    pub api_path: String,
    pub request_timeout: u64,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct Api {
    pub request_timeout_secs: u64,
}

impl ApplicationConfig {
    pub fn new(env_prefix: &str, prefix_separator: &str, separator: &str) -> anyhow::Result<Self> {
        let c = config::Config::builder()
            .add_source(
                Environment::with_prefix(env_prefix)
                    .prefix_separator(prefix_separator)
                    .separator(separator),
            )
            .set_override("config.env_prefix", env_prefix)?
            .build()?;

        let config = c.try_deserialize()?;

        Ok(config)
    }
}
