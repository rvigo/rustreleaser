use anyhow::Result;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{self, Write},
    path::Path,
};

pub struct Checksum {
    value: String,
}

impl Checksum {
    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn create(bytes: Vec<u8>, file_path: impl AsRef<Path>) -> Result<Self> {
        let mut file = File::options()
            .create(true)
            .write(true)
            .read(true)
            .open(file_path)?;
        file.write_all(&bytes)?;
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
    use tempdir::TempDir;

    #[test]
    fn should_create_checksum() -> Result<(), Box<dyn std::error::Error>> {
        let dir = TempDir::new("checksum")?;

        let file_path = dir.path().join("test.txt");

        let bytes = b"Hello, world!".to_vec();

        let checksum = Checksum::create(bytes, file_path)?;

        assert_eq!(
            checksum.value(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );

        dir.close()?;
        Ok(())
    }
}
