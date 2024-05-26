use std::cmp::Ordering;

use crate::github::tag::Tag;
use anyhow::{bail, Result};
use git2::Repository;
use itertools::Itertools;

pub fn get_current_tag() -> Result<Tag> {
    let repo = Repository::open(".")?;

    let binding = repo.tag_names(None)?;

    let ordered = binding.into_iter().sorted_by(|a, b| {
        let a = a.as_ref().unwrap().trim_start_matches('v');
        let b = b.as_ref().unwrap().trim_start_matches('v');

        let mut a_parts = a.split(|c| c == '.' || c == '-');
        let mut b_parts = b.split(|c| c == '.' || c == '-');

        loop {
            match (a_parts.next(), b_parts.next()) {
                (Some(a_part), Some(b_part)) => {
                    match (a_part.parse::<u32>(), b_part.parse::<u32>()) {
                        (Ok(a_num), Ok(b_num)) => match a_num.cmp(&b_num) {
                            Ordering::Equal => continue,
                            non_eq => return non_eq,
                        },
                        (Ok(_), Err(_)) => return Ordering::Less,
                        (Err(_), Ok(_)) => return Ordering::Greater,
                        (Err(_), Err(_)) => match a_part.cmp(b_part) {
                            Ordering::Equal => continue,
                            non_eq => return non_eq,
                        },
                    }
                }
                (Some(_), None) => return Ordering::Greater,
                (None, Some(_)) => return Ordering::Less,
                (None, None) => return Ordering::Equal,
            }
        }
    });

    let tag = match ordered.last().unwrap_or_default() {
        Some(tag) => tag,
        None => bail!(anyhow::anyhow!("No tags found")),
    };

    Ok(Tag::new(tag))
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

        assert_eq!(tag.value(), "v2.0.0");

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
        assert_eq!(tag.value(), "v1.1.10-beta");

        Ok(())
    }
}
