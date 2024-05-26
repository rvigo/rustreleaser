use anyhow::{Context, Result};
use flate2::GzBuilder;
use std::{fs::File, io::Write, path::PathBuf};
use tar::Builder;

use crate::build::compression::Compression;

pub fn compress_file(
    target_name: &str,
    target_path: PathBuf,
    zip_file_name: &str,
    compression: &Compression,
) -> Result<PathBuf> {
    log::debug!("using compression: {:?}", compression);
    match compression {
        Compression::TarGz => tar_gz(target_name, target_path, zip_file_name),
    }
}

fn tar_gz(target_name: &str, target_path: PathBuf, zip_file_name: &str) -> Result<PathBuf> {
    let zip_file_name = format!("{}.{}", zip_file_name, Compression::TarGz.extension());
    log::debug!(
        "compressing file: {} - {} at {}.",
        target_name,
        zip_file_name,
        target_path.display(),
    );

    let mut file = File::open(target_path).context("Cannot open file")?;
    let mut archive = Builder::new(Vec::new());

    archive.append_file(target_name, &mut file)?;
    let compressed_file = File::create(&zip_file_name).context("Cannot create the file")?;
    let mut encoder = GzBuilder::new()
        .filename(target_name)
        .write(compressed_file, flate2::Compression::Default);

    encoder
        .write_all(&archive.into_inner()?)
        .context("Cannot write to file")?;

    encoder.try_finish()?;

    Ok(PathBuf::from(zip_file_name))
}
