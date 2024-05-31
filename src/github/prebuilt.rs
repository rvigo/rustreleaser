use super::asset::Assets;
use crate::{
    brew::package::Package,
    build::Build,
    checksum::Checksum,
    compression::{compress_file, Compression},
    config::ReleaseConfig,
    cwd,
    git::{self, tag::Tag},
    github::{
        asset::Asset,
        asset_matrix::{AssetMatrix, AssetMatrixEntry},
        create_compressed_asset, get_release,
    },
};
use anyhow::Result;
use futures::future::join_all;
use itertools::Itertools;

pub async fn release(
    build: &Build,
    release_config: &ReleaseConfig,
    compression: &Compression,
) -> Result<Vec<Package>> {
    let tag = git::get_current_tag(cwd!())?;
    let matrix = build_matrix(build, release_config, compression, tag.to_owned())?;
    let assets = build_assets(&matrix)?;

    let release = get_release(release_config, &tag).await?;

    log::debug!("preparing to upload assets");
    let uploaded_assets_futures = release.upload_assets_raw(&assets, &tag);
    let checksum_assets_futures = assets
        .iter()
        .map(|asset| release.upload_checksum_asset_raw(&tag, &asset))
        .collect_vec();

    log::debug!("uploading assets");
    let uploaded_assets = join_all(uploaded_assets_futures).await;
    let _checksum_assets = join_all(checksum_assets_futures).await;

    let (success, _): (Vec<_>, Vec<_>) = uploaded_assets.into_iter().partition(Result::is_ok);
    let success = success.into_iter().map(Result::unwrap).collect::<Vec<_>>();

    log::debug!("creating packages");
    let packages: Vec<Package> = matrix
        .enrich(success)
        .iter()
        .cloned()
        .map(|e| e.into_package())
        .collect();

    Ok(packages)
}

pub fn build_assets(matrix: &AssetMatrix) -> Result<Assets> {
    let assets: Assets = Vec::<Asset>::from(matrix);

    Ok(assets)
}

fn build_matrix(
    build: &Build,
    release_config: &ReleaseConfig,
    compression: &Compression,
    tag: Tag,
) -> Result<AssetMatrix> {
    let prebuilt_items = build.to_owned().prebuilt.unwrap_or_default();
    let mut matrix: AssetMatrix = AssetMatrix::default();

    for prebuilt in prebuilt_items.into_iter() {
        let path = prebuilt.path.to_owned();
        if path.is_dir() {
            log::debug!("path is a directory, ignoring");
            continue;
        }
        // TODO fix this
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();
        log::debug!("creating matrix entry for {:#?}", name);
        let mut entry = AssetMatrixEntry::new(
            prebuilt.arch.unwrap(),
            prebuilt.os.unwrap(),
            &name,
            tag.name(),
            compression.to_owned(),
            true,
        );

        let full_name = format!(
            "{}-{}-{}-{}",
            name,
            tag.name(),
            entry.arch.to_string(),
            entry.os.to_string()
        );
        let compressed_file_path = compress_file(
            &name,
            path,
            &full_name,
            &release_config.archive.files,
            compression,
        )?;

        log::debug!("creating asset for {:#?}", full_name);
        let mut asset = create_compressed_asset(&full_name, compressed_file_path, compression);

        log::debug!("asset created: {:?}", asset);

        log::debug!("generating checksum for {}", full_name);
        let checksum = Checksum::try_from(&asset)
            .unwrap_or_else(|_| panic!("Failed to generate checksum for asset {:#?}", asset));

        asset.add_checksum(checksum.value());
        entry.set_asset(asset);
        matrix.push(entry);
        log::debug!("matrix entry created for {}", full_name)
    }

    Ok(matrix)
}
