pub mod asset;
mod asset_arch_os_matrix;
pub mod builder;
mod dto;
pub mod github_client;
mod handler;
mod macros;
mod multi;
mod prebuilt;
mod release;
mod request;
mod response;
mod single;

use self::release::Release;
use crate::{
    brew::package::Package, build::Build, compression::Compression, config::ReleaseConfig,
    git::tag::Tag, github::asset::Asset,
};
use anyhow::{bail, Result};
use asset_arch_os_matrix::AssetArchOsMatrix;
use builder::BuilderExecutor;
use std::{
    fs,
    path::{Path, PathBuf},
};

const SINGLE_TARGET_DIR: &str = "target/release";

pub async fn release(build: &Build, release_config: &ReleaseConfig) -> Result<Vec<Package>> {
    let packages = match build.target_type() {
        crate::build::TargetType::Multi => {
            log::debug!("Running multi target");
            multi::release(build, release_config).await?
        }
        crate::build::TargetType::Single => {
            log::debug!("Running single target");
            single::release(build, release_config).await?
        }
        crate::build::TargetType::PreBuilt => {
            log::debug!("Running prebuilt, ignoring build info");
            prebuilt::release(build, release_config, &release_config.compression).await?
        }
    };

    Ok(packages)
}

fn check_binary(name: &str, target: Option<String>) -> Result<()> {
    log::debug!("checking binary: {} - {:#?}", name, target);
    let binary_path = if let Some(target) = target {
        format!("target/{}/release/{}", target, name)
    } else {
        format!("target/release/{}", name)
    };

    if !PathBuf::from(binary_path).exists() {
        bail!(anyhow::anyhow!(
            "no release folder found, please run `cargo build --release`"
        ));
    }
    Ok(())
}

async fn do_create_release(release_config: &ReleaseConfig, tag: &Tag) -> Result<Release> {
    github_client::instance()
        .repo(&release_config.owner, &release_config.repo)
        .releases()
        .create()
        .tag(tag)
        .target_branch(&release_config.target_branch)
        .name(
            release_config
                .name
                .as_ref()
                .unwrap_or(&tag.name().to_owned()),
        )
        .draft(release_config.draft)
        .prerelease(release_config.prerelease)
        .body(&release_config.body)
        .execute()
        .await
}

async fn get_release_by_tag(release_config: &ReleaseConfig, tag: &Tag) -> Result<Release> {
    github_client::instance()
        .repo(&release_config.owner, &release_config.repo)
        .releases()
        .get_by_tag(tag)
        .await
}

async fn get_release(release_config: &ReleaseConfig, tag: &Tag) -> Result<Release> {
    let res = get_release_by_tag(release_config, tag).await;
    match res {
        Ok(release) => {
            log::info!("found release by tag: {:?}", tag);
            Ok(release)
        }
        Err(err) => {
            log::warn!(
                "cannot find a release by tag: {:?}, trying to create a new one",
                err
            );

            do_create_release(release_config, tag).await
        }
    }
}

fn create_compressed_asset(
    name: impl Into<String>,
    path: impl AsRef<Path>,
    compression: &Compression,
) -> Asset {
    Asset::new(format!("{}.{}", name.into(), compression.extension()), path)
}

fn generate_checksum_asset(asset: &Asset) -> Result<Asset> {
    if let Some(checksum) = &asset.checksum {
        let sha256_file_name = format!("{}.sha256", asset.name);

        let path = PathBuf::from(&sha256_file_name);
        fs::write(&path, format!("{}  {}", checksum, asset.name))?;

        let asset = Asset::new(&sha256_file_name, path);
        Ok(asset)
    } else {
        bail!(anyhow::anyhow!(
            "checksum is not available for asset {:#?}",
            asset
        ))
    }
}

type Assets = Vec<Asset>;

impl From<&AssetArchOsMatrix<'_>> for Assets {
    fn from(value: &AssetArchOsMatrix) -> Self {
        value
            .iter()
            .cloned()
            .filter_map(|entry| entry.asset)
            .collect()
    }
}

impl From<AssetArchOsMatrix<'_>> for Assets {
    fn from(value: AssetArchOsMatrix) -> Self {
        value
            .iter()
            .cloned()
            .filter_map(|entry| entry.asset)
            .collect()
    }
}
