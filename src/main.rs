mod arch_os_matrix;
mod brew;
mod build;
mod cargo;
mod checksum;
mod compress;
mod config;
mod git;
mod github;
mod http;
mod logger;
mod target;
mod template;

use anyhow::Result;
use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    logger::init()?;

    log::info!("Starting");

    let config = Config::load().await?;
    let build_info = config.build;
    let release_info = config.release;

    log::info!("Building");
    if !build_info.has_prebuilt() {
        cargo::build(&build_info).await?;
    }

    log::info!("Creating release");
    let packages = github::release(&build_info, &release_info).await?;

    if config.brew.is_some() {
        log::info!("Creating brew formula");
        brew::release(config.brew.unwrap(), packages).await?;
    }

    Ok(())
}
