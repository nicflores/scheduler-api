use std::env;

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Deserialize, serde::Serialize, Debug, PartialEq, Clone)]
pub struct AppConfig {
    database_url: String,
    api_key: String,
    log_level: String,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "local".into());
        Config::builder()
            .add_source(File::with_name(&format!("src/config/{}.toml", run_mode)).required(false))
            .add_source(Environment::with_prefix("APP"))
            .build()
            .unwrap()
            .try_deserialize()
    }
}
