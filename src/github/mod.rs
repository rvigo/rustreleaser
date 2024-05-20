mod asset;
mod asset_arch_os_matrix_entry;
pub mod builder;
pub mod dto;
pub mod github_client;
mod handler;
mod macros;
mod release;
mod request;
mod response;
pub mod tag;

use self::{
    asset::UploadedAsset, asset_arch_os_matrix_entry::AssetArchOsMatrixEntry,
    builder::BuilderExecutor, release::Release, tag::Tag,
};
use crate::{
    arch_os_matrix::ArchOsMatrix,
    brew::package::Package,
    build::{arch::Arch, compression::Compression, os::Os, prebuilt::PreBuiltAsset, Build},
    checksum,
    config::ReleaseConfig,
    git,
    github::asset::Asset,
};
use anyhow::{bail, Result};
use flate2::write::GzEncoder;
use itertools::Itertools;
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    vec,
};
use tar::Builder;

const SINGLE_TARGET_DIR: &str = "target/release";

pub async fn release(build_info: &Build, release_info: &ReleaseConfig) -> Result<Vec<Package>> {
    let packages = if let Some(pb) = &build_info.prebuilt {
        log::debug!("Running prebuilt, ignoring build info");
        prebuilt(pb, release_info, &build_info.compression).await?
    } else if build_info.is_multi_target() {
        log::debug!("Running multi target");
        multi(build_info, release_info).await?
    } else {
        log::debug!("Running single target");
        single(build_info, release_info).await?
    };

    Ok(packages)
}

async fn single(build_info: &Build, release_info: &ReleaseConfig) -> Result<Vec<Package>> {
    // validate binary
    check_binary(&build_info.binary.to_owned(), None)?;

    let tag = git::get_current_tag()?;

    // calculate full binary name
    let binary_name = format!(
        "{}_{}.{}",
        build_info.binary,
        tag.value(),
        build_info.compression.extension()
    );

    log::debug!("binary name: {}", binary_name);

    // zip binary
    log::debug!("zipping binary");
    zip_file(
        &build_info.binary.to_owned(),
        &binary_name.to_owned(),
        PathBuf::from(format!("{}/{}", SINGLE_TARGET_DIR, build_info.binary)),
    )?;

    let path = PathBuf::from(binary_name.to_owned());

    // create an asset
    log::debug!("creating asset");
    let mut asset = create_asset(binary_name, path);

    // generate a checksum value
    log::debug!("generating checksum");
    let checksum = generate_checksum(&asset)?;

    // add checksum to asset
    log::debug!("adding checksum to asset");
    asset.add_checksum(checksum);

    // create release
    log::debug!("getting/creating release");
    let release = get_release(release_info, &tag).await?;

    // upload to release
    log::debug!("uploading asset");
    let uploaded_assets = match release.upload_assets(vec![asset], &tag).await {
        Ok(uploaded_assets) => uploaded_assets,
        Err(e) => {
            log::error!("Failed to upload asset {:#?}", e);
            bail!(anyhow::anyhow!("Failed to upload asset"))
        }
    };

    // return a package with the asset url and checksum value
    let packages: Vec<Package> = uploaded_assets
        .iter()
        .map(|asset| package_asset(asset, None, None))
        .collect();

    Ok(packages)
}

async fn multi(build: &Build, release_config: &ReleaseConfig) -> Result<Vec<Package>> {
    let tag = git::get_current_tag()?;

    let archs = build.arch.to_owned().unwrap_or_default();
    let os = build.os.to_owned().unwrap_or_default();
    let mut matrix: Vec<AssetArchOsMatrixEntry> = Vec::new();

    for arch in &archs {
        for os in &os {
            let binary = build.binary.to_owned();
            check_binary(
                &binary,
                Some(format!("{}-{}", &arch.to_string(), &os.to_string())),
            )?;

            let mut entry =
                AssetArchOsMatrixEntry::new(arch, os, binary, tag.value(), &build.compression);

            let target = format!("{}-{}", &arch.to_string(), &os.to_string());

            log::debug!("zipping binary for {}", target);
            let entry_name = entry.name.to_owned();

            // zip binary
            zip_file(
                &build.binary.to_owned(),
                &entry_name,
                PathBuf::from(format!("target/{}/release/{}", target, build.binary)),
            )?;

            // create an asset
            let mut asset = Asset::new(&entry.name, &entry_name);

            // generate a checksum value
            let checksum = generate_checksum(&asset)
                .unwrap_or_else(|_| panic!("Failed to generate checksum for asset {:#?}", asset));

            // add checksum to asset
            asset.add_checksum(checksum);

            entry.set_asset(asset);
            matrix.push_entry(entry);
        }
    }

    let release = get_release(release_config, &tag).await?;

    let assets: Vec<Asset> = matrix
        .iter()
        .cloned()
        .filter_map(|entry| entry.asset)
        .collect();

    // upload to release
    let uploaded_assets = release.upload_assets(assets, &tag).await?;

    let packages: Vec<Package> = matrix
        .into_iter()
        .map(|entry| {
            let asset = uploaded_assets
                .iter()
                .find(|asset| asset.name == entry.name)
                .expect("asset not found");

            package_asset(asset, Some(entry.os), Some(entry.arch))
        })
        .collect();

    Ok(packages)
}

pub async fn prebuilt(
    prebuilt_items: &Vec<PreBuiltAsset>,
    release_config: &ReleaseConfig,
    compression: &Compression,
) -> Result<Vec<Package>> {
    let mut matrix: Vec<AssetArchOsMatrixEntry> = Vec::new();

    let tag = git::get_current_tag()?;
    for prebuilt in prebuilt_items.iter() {
        let path = prebuilt.path.to_owned();
        if path.is_dir() {
            log::info!("path is a directory, ignoring");
            continue;
        }
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();
        log::debug!("creating matrix entry for {:#?}", name);
        let mut entry = AssetArchOsMatrixEntry::new(
            prebuilt.arch.as_ref().unwrap(),
            prebuilt.os.as_ref().unwrap(),
            &name,
            tag.value(),
            compression,
        );

        log::debug!("zipping binary for {:#?}", name);
        zip_file(&name, &name, path.to_owned())?;

        log::debug!("creating asset for {:#?}", name);
        let mut asset = create_asset(&name, path);

        log::debug!("generating checksum for {:#?}", name);
        let checksum = generate_checksum(&asset)
            .unwrap_or_else(|_| panic!("Failed to generate checksum for asset {:#?}", asset));

        asset.add_checksum(checksum);
        entry.set_asset(asset);
        matrix.push_entry(entry);
    }

    let release = get_release(release_config, &tag).await?;

    let assets: Vec<Asset> = matrix
        .iter()
        .cloned()
        .filter_map(|entry| entry.asset)
        .collect();
    // upload to release
    log::debug!("uploading asset");

    let uploaded_assets = match release.upload_assets(assets, &tag).await {
        Ok(uploaded_assets) => uploaded_assets,
        Err(e) => {
            log::error!("Failed to upload asset {:#?}", e);
            bail!(anyhow::anyhow!("Failed to upload asset"))
        }
    };

    println!(
        "{:#?}",
        uploaded_assets
            .iter()
            .map(|u| u.name.to_owned())
            .collect_vec()
    );
    println!(
        "{:#?}",
        matrix.iter().map(|m| m.name.to_owned()).collect_vec()
    );

    let packages: Vec<Package> = matrix
        .into_iter()
        .map(|entry| {
            let asset = uploaded_assets
                .iter()
                .find(|asset| asset.name == entry.asset.as_ref().expect("asset not present").name)
                .expect("asset not found");

            package_asset(asset, Some(entry.os), Some(entry.arch))
        })
        .collect();

    Ok(packages)
}

fn zip_file(binary_name: &str, full_binary_name: &str, binary_path: PathBuf) -> Result<()> {
    log::debug!(
        "zipping file: {} - {} at {}",
        binary_name,
        full_binary_name,
        binary_path.display()
    );
    let mut file = File::open(binary_path)?;
    let mut archive = Builder::new(Vec::new());

    archive.append_file(binary_name, &mut file)?;

    let compressed_file = File::create(full_binary_name)?;
    let mut encoder = GzEncoder::new(compressed_file, flate2::Compression::Default);
    encoder.write_all(&archive.into_inner()?)?;

    encoder.try_finish()?;

    Ok(())
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
        .name(&release_config.name)
        .draft(release_config.draft)
        .prerelease(release_config.prerelease)
        .body(release_config.body.as_ref().unwrap_or(&String::new()))
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

fn create_asset(name: impl Into<String>, path: impl AsRef<Path>) -> Asset {
    Asset::new(name, path)
}

fn generate_checksum(asset: &Asset) -> Result<String> {
    let checksum = checksum::create(&asset.path)?;
    Ok(checksum)
}

fn generate_checksum_asset(asset: &Asset) -> Result<Asset> {
    if let Some(checksum) = &asset.checksum {
        let sha256_file_name = format!("{}.sha256", asset.name);

        let path = PathBuf::from(&sha256_file_name);
        fs::write(&path, format!("{}  {}", checksum, asset.name))?;

        let asset = create_asset(&sha256_file_name, path);
        Ok(asset)
    } else {
        bail!(anyhow::anyhow!(
            "checksum is not available for asset {:#?}",
            asset
        ))
    }
}

fn package_asset(asset: &UploadedAsset, os: Option<&Os>, arch: Option<&Arch>) -> Package {
    Package::new(
        asset.name.to_owned(),
        os.map(|os| os.to_owned()),
        arch.map(|arch| arch.to_owned()),
        asset.url.to_owned(),
        asset.checksum.to_owned(),
    )
}
