use config::Environment;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Default)]
#[allow(dead_code)]
pub struct ApplicationConfig {
    pub config: ConfigInfo,
    pub port: Option<u16>,
    pub database: Database,
    pub event_provider: EventProvider,
    pub api: Api,
}

#[derive(Clone, Debug, Deserialize, Default)]
#[allow(dead_code)]
pub struct ConfigInfo {
    pub env_prefix: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct Database {
    pub url: Option<String>,
    pub max_connections: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct EventProvider {
    pub url: Option<String>,
    pub api_path: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct Api {
    pub request_timeout_secs: Option<u64>,
}

const APP_CONFIG_PREFIX_SEPARATOR: &'static str = "__";
const APP_CONFIG_SEPARATOR: &'static str = "__";

impl ApplicationConfig {
    pub fn new(env_prefix: &str) -> anyhow::Result<Self> {
        let c = config::Config::builder()
            .add_source(
                Environment::with_prefix(env_prefix)
                    .prefix_separator(APP_CONFIG_PREFIX_SEPARATOR)
                    .separator(APP_CONFIG_SEPARATOR),
            )
            .set_override("config.env_prefix", env_prefix)?
            .build()?;

        let config = c.try_deserialize()?;

        Ok(config)
    }
}
