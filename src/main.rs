mod brew;
mod build;
mod cargo;
mod checksum;
mod compression;
mod config;
mod git;
mod github;
mod http;
mod logger;

use anyhow::{Context, Result};
use build::TargetType;
use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    logger::init()?;

    log::info!("Starting");
    let config = Config::load().await.context("Cannot load config file")?;
    let build_info = config.build;
    let release_info = config.release;

    if build_info.target_type() != TargetType::PreBuilt {
        log::info!("Building");
        cargo::build(&build_info).await?;
    }

    log::info!("Creating release");
    let packages = github::release(&build_info, &release_info).await?;

    if config.brew.is_some() {
        log::info!("Creating brew formula");
        brew::publish(config.brew.unwrap(), packages).await?;
    }

    Ok(())
}
