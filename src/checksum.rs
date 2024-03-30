use anyhow::Result;
use sha2::{Digest, Sha256};
use std::{fs::File, io, path::Path};

pub fn create(path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    let mut file = File::open(path)?;

    let mut hasher = Sha256::new();
    let _ = io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();

    let encoded = hex::encode(hash);

    Ok(encoded)
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

        let checksum = create(&file_path)?;

        assert_eq!(
            checksum,
            "d9014c4624844aa5bac314773d6b689ad467fa4e1d1a50a1b8a99d5a95f72ff5"
        );

        dir.close()?;
        Ok(())
    }

    #[test]
    fn should_return_err_with_nonexistent_file() {
        let result = create("nonexistent.txt");

        assert!(result.is_err());
    }
}
