pub mod committer;
pub mod tag;

use anyhow::{bail, Context, Result};
use git2::Repository;
use semver::Version;
use tag::Tag;

pub fn get_current_tag() -> Result<Tag> {
    let repo = Repository::open(".").context("Cannot read repo info")?;
    let tags = repo.tag_names(None)?;

    let mut tags = tags
        .iter()
        .map(|tag| Tag::new(tag.unwrap_or_default()))
        .filter_map(|tag| {
            Version::parse(tag.strip_v_prefix())
                .ok()
                .map(|version| (tag, version))
        })
        .collect::<Vec<_>>();
    tags.sort_by(|(_, a), (_, b)| a.cmp(b));
    let sorted = tags.into_iter().map(|(tag, _)| tag).collect::<Vec<Tag>>();

    let tag = match sorted.last() {
        Some(tag) => tag,
        None => bail!(anyhow::anyhow!("No tags found")),
    };

    Ok(tag.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::Repository;
    use std::{
        fs::{self},
        path::Path,
        process::Command,
    };
    use tempdir::TempDir;

    fn init_gitconfig() {
        let _ = Command::new("git")
            .args(&["config", "--local", "user.name", "Test User"])
            .output()
            .expect("Failed to set user name for test");

        let _ = Command::new("git")
            .args(&["config", "--local", "user.email", "test@example.com"])
            .output()
            .expect("Failed to set email for test");

        let _ = Command::new("git")
            .args(&[
                "config",
                "--local",
                "http.https://github.com/.extraheader",
                "AUTHORIZATION: basic ***",
            ])
            .output()
            .expect("Failed to set extraheader for test");
    }

    #[test]
    fn test_get_current_tag() -> Result<(), Box<dyn std::error::Error>> {
        let dir = TempDir::new("git")?;
        init_gitconfig();

        let repo = Repository::init(dir.path())?;

        fs::write(dir.path().join("test.txt"), "Hello, world!")?;

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

        let obj = &repo.revparse_single("HEAD")?;
        repo.tag("v1.0.0", obj, &signature, "Version 1.0.0", false)?;

        println!("creating commit 2");

        let oid = index.write_tree()?;
        let parent_commit = repo.head()?.peel_to_commit()?;

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "commit",
            &repo.find_tree(oid)?,
            &[&parent_commit],
        )?;

        println!("creating tag 2");
        let obj = &repo.revparse_single("HEAD")?;
        repo.tag("v2.0.0", obj, &signature, "Version 2.0.0", false)?;

        std::env::set_current_dir(dir.path())?;

        let tag = get_current_tag()?;

        assert_eq!(tag.name(), "v2.0.0");

        dir.close()?;

        Ok(())
    }

    #[test]
    fn test_get_current_tag_no_tags() -> Result<(), Box<dyn std::error::Error>> {
        let dir = TempDir::new("git")?;
        init_gitconfig();

        Repository::init(dir.path())?;

        std::env::set_current_dir(dir.path())?;

        let result = get_current_tag();

        assert!(result.is_err());

        dir.close()?;

        Ok(())
    }

    #[test]
    fn test_handle_different_formats() -> Result<(), Box<dyn std::error::Error>> {
        let dir = TempDir::new("git")?;
        init_gitconfig();

        let repo = Repository::init(dir.path())?;

        fs::write(dir.path().join("test.txt"), "Hello, world!")?;

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

        let obj = &repo.revparse_single("HEAD")?;
        repo.tag("1.1.8", obj, &signature, "Version 1.1.8", false)?;

        println!("creating commit 2");
        let oid = index.write_tree()?;
        let parent_commit = repo.head()?.peel_to_commit()?;

        println!("creating tag 2");
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "commit",
            &repo.find_tree(oid)?,
            &[&parent_commit],
        )?;

        let obj = &repo.revparse_single("HEAD")?;
        repo.tag("v1.1.9", obj, &signature, "Version 1.1.9", false)?;

        println!("creating commit 3");
        let oid = index.write_tree()?;
        let parent_commit = repo.head()?.peel_to_commit()?;

        println!("creating tag 2");
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "commit",
            &repo.find_tree(oid)?,
            &[&parent_commit],
        )?;

        let obj = &repo.revparse_single("HEAD")?;
        repo.tag("v1.1.10-beta", obj, &signature, "Version 1.1.10", false)?;
        std::env::set_current_dir(dir.path())?;

        let tag = get_current_tag()?;
        assert_eq!(tag.name(), "v1.1.10-beta");

        Ok(())
    }
}
