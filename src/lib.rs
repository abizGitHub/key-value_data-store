pub mod app_server;
pub mod services;
use config::{Config, ConfigError, Environment, File};
use once_cell::sync::Lazy;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    db_file: String,
}
// APP_ENV=dev cargo run
// APP_ENV=test cargo test
impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "dev".into());

        let builder = Config::builder()
            .add_source(File::with_name(&format!("config/{}.yml", env)).required(true))
            .add_source(Environment::with_prefix("APP").separator("__"));
        builder.build()?.try_deserialize()
    }
}

static SETTING: Lazy<Settings> = Lazy::new(|| Settings::new().unwrap());
