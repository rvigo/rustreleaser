mod brew;
mod checksum;
mod compression;
mod config;
mod git;
mod github;
mod http;
mod logger;

use anyhow::{Context, Result};
use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    logger::init()?;

    log::info!("Starting");
    let config = Config::load().await.context("Cannot load config file")?;

    let release_config = config.release;

    log::info!("Creating release and getting checksum");
    let release_output = github::release(&release_config).await?;

    log::info!("Creating brew formula");
    brew::publish(config.brew, release_output)
        .await
        .context("Cannot publish the brew formula")?;

    Ok(())
}
