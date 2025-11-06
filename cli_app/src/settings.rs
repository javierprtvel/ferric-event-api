use config::{Config, Environment, File};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Default)]
pub struct Database {
    pub url: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct Logging {
    pub log_level: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct ConfigInfo {
    pub location: Option<String>,
    pub env_prefix: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct Settings {
    #[serde(default)]
    pub config: ConfigInfo,
    #[serde(default)]
    pub database: Database,
    #[serde(default)]
    pub logging: Logging,
}

impl Settings {
    pub fn new(location: Option<&str>, env_prefix: &str) -> anyhow::Result<Self> {
        let mut builder = Config::builder();
        if let Some(location) = location {
            builder = builder.add_source(File::with_name(location));
        }

        let s = builder
            .add_source(
                Environment::with_prefix(env_prefix)
                    .prefix_separator("__")
                    .separator("__"),
            )
            //  store the config file location and other parameters required to be able to reload the configuration later
            .set_override("config.location", location)?
            .set_override("config.env_prefix", env_prefix)?
            .build()?;

        let settings = s.try_deserialize()?;

        Ok(settings)
    }
}
