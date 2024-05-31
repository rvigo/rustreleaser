use crate::{
    brew::package::Package,
    build::Build,
    checksum::Checksum,
    compression::{compress_file, Compression},
    config::ReleaseConfig,
    cwd, git,
    github::{
        asset::Asset,
        asset_matrix::{AssetMatrix, AssetMatrixEntry},
        create_compressed_asset, get_release,
    },
};
use anyhow::{bail, Result};

pub async fn release(
    build: &Build,
    release_config: &ReleaseConfig,
    compression: &Compression,
) -> Result<Vec<Package>> {
    let prebuilt_items = build.to_owned().prebuilt.unwrap_or_default();
    let mut matrix: AssetMatrix = AssetMatrix::default();

    let tag = git::get_current_tag(cwd!())?;
    for prebuilt in prebuilt_items.iter() {
        let path = prebuilt.path.to_owned();
        if path.is_dir() {
            log::debug!("path is a directory, ignoring");
            continue;
        }
        // TODO fix this
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();
        log::debug!("creating matrix entry for {:#?}", name);
        let mut entry = AssetMatrixEntry::new(
            prebuilt.arch.as_ref().unwrap(),
            prebuilt.os.as_ref().unwrap(),
            &name,
            tag.name(),
            compression,
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

        log::debug!("generating checksum for {:#?}", full_name);
        let checksum = Checksum::try_from(&asset)
            .unwrap_or_else(|_| panic!("Failed to generate checksum for asset {:#?}", asset));

        asset.add_checksum(checksum.value());
        entry.set_asset(asset);
        matrix.push(entry);
    }

    let release = get_release(release_config, &tag).await?;

    let assets: Vec<Asset> = Vec::<Asset>::from(&matrix);

    log::debug!("uploading asset");
    let uploaded_assets = match release.upload_assets(assets, &tag).await {
        Ok(uploaded_assets) => uploaded_assets,
        Err(e) => {
            log::error!("Failed to upload asset {:#?}", e);
            bail!(anyhow::anyhow!("Failed to upload asset"))
        }
    };

    let packages: Vec<Package> = matrix
        .enrich(uploaded_assets)
        .iter()
        .cloned()
        .map(|e| e.into_package())
        .collect();

    Ok(packages)
}
