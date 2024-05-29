use crate::{
    brew::package::Package,
    build::Build,
    checksum::Checksum,
    compression::compress_file,
    config::ReleaseConfig,
    git,
    github::{check_binary, create_compressed_asset, get_release, SINGLE_TARGET_DIR},
};
use anyhow::{bail, Result};
use std::path::PathBuf;

pub async fn release(build: &Build, release_config: &ReleaseConfig) -> Result<Vec<Package>> {
    check_binary(&build.binary.to_owned(), None)?;

    let tag = git::get_current_tag()?;

    let binary_name = format!(
        "{}_{}.{}",
        build.binary,
        tag.name(),
        release_config.compression.extension()
    );

    log::debug!("compressing binary");
    let compressed_file_path = compress_file(
        &build.binary.to_owned(),
        PathBuf::from(format!("{}/{}", SINGLE_TARGET_DIR, build.binary)),
        &binary_name.to_owned(),
        &release_config.archive,
        &release_config.compression,
    )?;

    log::debug!("creating asset");
    let mut asset = create_compressed_asset(
        &binary_name,
        compressed_file_path,
        &release_config.compression,
    );

    log::debug!("generating checksum");
    let checksum = Checksum::try_from(&asset)?;

    log::debug!("adding checksum to asset");
    asset.add_checksum(checksum.value());

    log::debug!("getting/creating release");
    let release = get_release(release_config, &tag).await?;

    log::debug!("uploading asset");
    let uploaded_assets = match release.upload_assets(vec![asset], &tag).await {
        Ok(uploaded_assets) => uploaded_assets,
        Err(e) => {
            log::error!("Failed to upload asset {:#?}", e);
            bail!(anyhow::anyhow!("Failed to upload asset"))
        }
    };

    let packages: Vec<Package> = uploaded_assets
        .iter()
        .map(|asset| {
            Package::new(
                asset.name.to_owned(),
                None,
                None,
                asset.url.to_owned(),
                asset.checksum.to_owned(),
                false,
            )
        })
        .collect();

    Ok(packages)
}
