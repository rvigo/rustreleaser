use anyhow::{bail, Result};
use git2::Repository;

use crate::github::tag::Tag;

pub fn get_current_tag() -> Result<Tag> {
    let repo = Repository::open(".")?;

    let binding = repo.tag_names(None)?;

    let tag = match binding.into_iter().rev().last().unwrap_or_default() {
        Some(tag) => tag,
        None => bail!(anyhow::anyhow!("No tags found")),
    };

    Ok(Tag::new(tag))
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::Repository;
    use std::{fs, path::Path};
    use tempdir::TempDir;

    #[test]
    fn test_get_current_tag() -> Result<(), Box<dyn std::error::Error>> {
        // Create a temporary directory.
        let dir = TempDir::new("git")?;

        // Initialize a new repository in the temporary directory.
        let repo = Repository::init(dir.path())?;

        // Create a new file in the repository.
        fs::write(dir.path().join("test.txt"), "Hello, world!")?;

        // Stage and commit the new file.
        let mut index = repo.index()?;
        index.add_path(Path::new("test.txt"))?;
        let oid = index.write_tree()?;
        let signature = repo.signature()?;
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &repo.find_tree(oid)?,
            &[],
        )?;

        // Create a new tag in the repository.
        let obj = &repo.revparse_single("HEAD")?;
        repo.tag("v1.0.0", obj, &signature, "Version 1.0.0", false)?;

        // Change the current directory to the repository directory.
        std::env::set_current_dir(dir.path())?;

        // Get the current tag.
        let tag = get_current_tag()?;

        // The tag should be "v1.0.0".
        assert_eq!(tag.value(), "v1.0.0");

        // Delete the temporary directory.
        dir.close()?;

        Ok(())
    }

    #[test]
    fn test_get_current_tag_no_tags() -> Result<(), Box<dyn std::error::Error>> {
        // Create a temporary directory.
        let dir = TempDir::new("git")?;

        // Initialize a new repository in the temporary directory.
        Repository::init(dir.path())?;

        // Change the current directory to the repository directory.
        std::env::set_current_dir(dir.path())?;

        // Try to get the current tag.
        let result = get_current_tag();

        // The function should return an error because there are no tags.
        assert!(result.is_err());

        // Delete the temporary directory.
        dir.close()?;

        Ok(())
    }
}
