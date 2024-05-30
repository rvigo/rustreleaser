use crate::github::asset::Asset;
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::{fs::File, io, path::Path};

pub struct Checksum {
    value: String,
}

impl Checksum {
    pub fn new(asset_path: impl AsRef<Path>) -> Result<Self> {
        Self::create(asset_path)
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    fn create(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let mut file = File::open(path).context("Cannot open file")?;

        let mut hasher = Sha256::new();
        let _ = io::copy(&mut file, &mut hasher)?;
        let hash = hasher.finalize();

        let encoded = hex::encode(hash);

        Ok(Checksum { value: encoded })
    }
}

impl TryFrom<&Asset> for Checksum {
    type Error = anyhow::Error;

    fn try_from(asset: &Asset) -> Result<Self> {
        Checksum::new(&asset.path)
    }
}

impl TryFrom<Asset> for Checksum {
    type Error = anyhow::Error;

    fn try_from(asset: Asset) -> Result<Self> {
        Checksum::new(asset.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempdir::TempDir;

    #[test]
    fn should_create_checksum() -> Result<(), Box<dyn std::error::Error>> {
        let dir = TempDir::new("checksum")?;

        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path)?;
        writeln!(file, "Hello, world!")?;

        let checksum = Checksum::new(&file_path)?;

        assert_eq!(
            checksum.value(),
            "d9014c4624844aa5bac314773d6b689ad467fa4e1d1a50a1b8a99d5a95f72ff5"
        );

        dir.close()?;
        Ok(())
    }

    #[test]
    fn should_return_err_with_nonexistent_file() {
        let result = Checksum::new("nonexistent.txt");

        assert!(result.is_err());
    }
}
