use anyhow::{Context, Result};
use flate2::GzBuilder;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Write, path::PathBuf};
use tar::Builder;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum Compression {
    #[default]
    TarGz,
}

impl Compression {
    pub fn extension(&self) -> &str {
        match self {
            Compression::TarGz => "tar.gz",
        }
    }
}

pub fn compress_file(
    target_name: &str,
    target_path: PathBuf,
    compressed_file_name: &str,
    extra_files: &Option<Vec<String>>,
    compression: &Compression,
) -> Result<PathBuf> {
    match compression {
        Compression::TarGz => tar_gz(target_name, target_path, compressed_file_name, extra_files),
    }
}

fn tar_gz(
    target_name: &str,
    target_path: PathBuf,
    compressed_file_name: &str,
    extra_files: &Option<Vec<String>>,
) -> Result<PathBuf> {
    let compressed_file_name_with_extension = format!(
        "{}.{}",
        compressed_file_name,
        Compression::TarGz.extension()
    );
    log::debug!(
        "compressing file: {} - {} at {}.",
        target_name,
        compressed_file_name_with_extension,
        target_path.display(),
    );

    let mut file = File::open(target_path).context("Cannot open file")?;
    let mut archive = Builder::new(Vec::new());

    archive.append_file(target_name, &mut file)?;

    if let Some(extra) = extra_files {
        for path in extra {
            let glob = glob::glob(path).context("Cannot read glob pattern")?;

            for entry in glob {
                let path = entry.context("Cannot get path")?;

                if path.is_file() {
                    log::debug!("archiving file: {}", path.display());
                    archive.append_path(path).context("Cannot archive file")?;
                } else if path.is_dir() {
                    log::debug!("archiving dir: {}", path.display());
                    archive
                        .append_dir(&path, ".")
                        .context("Cannot archive directory")?;
                }
            }
        }
    }

    let compressed_file = File::create(&compressed_file_name_with_extension)
        .context("Cannot create the compressed file")?;
    let mut encoder = GzBuilder::new()
        .filename(target_name)
        .write(compressed_file, flate2::Compression::Default);

    encoder
        .write_all(&archive.into_inner()?)
        .context("Cannot write to file")?;

    encoder.try_finish()?;

    Ok(PathBuf::from(compressed_file_name_with_extension))
}
