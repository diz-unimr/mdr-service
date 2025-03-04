use config::{Config, ConfigError, Environment, File};
use serde_derive::Deserialize;

#[derive(Default, Debug, Deserialize, Clone)]
pub(crate) struct App {
    pub(crate) log_level: String,
}

#[derive(Default, Deserialize, Clone)]
pub(crate) struct Database {
    pub(crate) url: String,
    pub(crate) max_connections: Option<u32>,
    pub(crate) timeout: Option<u64>,
}

#[derive(Default, Deserialize, Clone)]
pub(crate) struct AppConfig {
    pub(crate) app: App,
    pub(crate) database: Database,
}

impl AppConfig {
    pub(crate) fn new() -> Result<Self, ConfigError> {
        Config::builder()
            // default config from file
            .add_source(File::with_name("app.yaml"))
            // override values from environment variables
            .add_source(Environment::default().separator("."))
            .build()?
            .try_deserialize()
    }
}
