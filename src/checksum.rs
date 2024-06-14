use anyhow::Result;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{self},
};

pub struct Checksum {
    value: String,
}

impl Checksum {
    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn create(mut file: &File) -> Result<Self> {
        let mut hasher = Sha256::new();
        let _ = io::copy(&mut file, &mut hasher)?;
        let hash = hasher.finalize();

        let encoded = hex::encode(hash);

        Ok(Checksum { value: encoded })
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
        File::create(&file_path)?.write_all(b"Hello, world!")?;

        let file = File::open(&file_path)?;
        let checksum = Checksum::create(&file)?;

        assert_eq!(
            checksum.value(),
            "315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3"
        );

        dir.close()?;
        Ok(())
    }
}
