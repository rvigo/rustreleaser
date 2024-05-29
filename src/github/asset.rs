use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Asset {
    pub name: String,
    pub path: PathBuf,
    pub checksum: Option<String>,
}

impl Asset {
    pub fn new(name: impl Into<String>, path: impl AsRef<Path>) -> Self {
        Self {
            name: name.into(),
            path: path.as_ref().to_path_buf(),
            checksum: None,
        }
    }

    pub fn add_checksum(&mut self, checksum: impl Into<String>) {
        self.checksum = Some(checksum.into());
    }
}

#[derive(Debug, Clone)]
pub struct UploadedAsset {
    pub name: String,
    pub url: String,
    pub checksum: String,
}

impl UploadedAsset {
    pub fn new(name: String, url: String, checksum: String) -> Self {
        Self {
            name,
            url,
            checksum,
        }
    }
}
