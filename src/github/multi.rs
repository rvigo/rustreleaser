use super::{
    asset_arch_os_matrix::{AssetArchOsMatrix, AssetArchOsMatrixEntry},
    check_binary, create_compressed_asset, get_release, Assets,
};
use crate::{
    brew::package::Package, build::Build, checksum::Checksum, compression::compress_file,
    config::ReleaseConfig, git,
};
use anyhow::Result;
use std::path::PathBuf;

pub async fn release(build: &Build, release_config: &ReleaseConfig) -> Result<Vec<Package>> {
    let tag = git::get_current_tag()?;

    let archs = build.arch.to_owned().unwrap_or_default();
    let os = build.os.to_owned().unwrap_or_default();
    let mut matrix: AssetArchOsMatrix = AssetArchOsMatrix::default();

    for arch in &archs {
        for os in &os {
            let binary = build.binary.to_owned();
            check_binary(
                &binary,
                Some(format!("{}-{}", &arch.to_string(), &os.to_string())),
            )?;

            let mut entry = AssetArchOsMatrixEntry::new(
                arch,
                os,
                binary,
                tag.name(),
                &release_config.compression,
                false,
            );

            let target = format!("{}-{}", &arch.to_string(), &os.to_string());

            let entry_name = entry.name.to_owned();

            let compressed_file_path = compress_file(
                &build.binary.to_owned(),
                PathBuf::from(format!("target/{}/release/{}", target, build.binary)),
                &entry_name,
                &release_config.archive,
                &release_config.compression,
            )?;

            let mut asset = create_compressed_asset(
                &entry.name,
                compressed_file_path,
                &release_config.compression,
            );
            let checksum = Checksum::try_from(&asset)
                .unwrap_or_else(|_| panic!("Failed to generate checksum for asset {:#?}", asset));

            asset.add_checksum(checksum.value());
            entry.set_asset(asset);
            matrix.push(entry);
        }
    }

    let release = get_release(release_config, &tag).await?;

    let assets: Assets = Assets::from(&matrix);

    let uploaded_assets = release.upload_assets(assets, &tag).await?;

    let packages: Vec<Package> = matrix
        .enrich(uploaded_assets)
        .iter()
        .cloned()
        .map(|e| e.into_package())
        .collect();

    Ok(packages)
}
