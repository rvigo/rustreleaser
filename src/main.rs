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
        cargo::build(&build_info)
            .await
            .context("Cannot build the project")?;
    }

    log::info!("Creating release");
    let packages = github::release(&build_info, &release_info)
        .await
        .context("Cannot create the github release")?;

    if let Some(brew) = config.brew {
        log::info!("Creating brew formula");
        brew::publish(brew, packages)
            .await
            .context("Cannot publish the brew formula")?;
    }

    Ok(())
}
