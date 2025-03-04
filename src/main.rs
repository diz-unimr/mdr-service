mod concept;
mod config;
mod module;
mod server;

use crate::config::AppConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // config
    let config = AppConfig::new().expect("Failed to load config");

    // run
    server::serve(config).await?;

    Ok(())
}
