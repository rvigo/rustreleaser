use anyhow::Result;
use sha2::{Digest, Sha256};
use std::io::{self, Read};

pub struct Checksum {
    value: String,
}

impl Checksum {
    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn create(mut reader: impl Read) -> Result<Self> {
        let mut hasher = Sha256::new();
        let _ = io::copy(&mut reader, &mut hasher)?;
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

        let bytes = b"Hello, world!".to_vec();
        let cursor = io::Cursor::new(bytes);

        let checksum = Checksum::create(cursor)?;

        assert_eq!(
            checksum.value(),
            "315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3"
        );

        dir.close()?;
        Ok(())
    }
}
