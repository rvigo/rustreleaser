pub mod committer;
pub mod tag;

use anyhow::{bail, Context, Result};
use git2::Repository;
use semver::Version;
use std::path::Path;
use tag::Tag;

pub fn get_current_tag(repo_path: impl AsRef<Path>) -> Result<Tag> {
    let repo = Repository::open(repo_path).context("Cannot read repo info")?;
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

// Get the current working directory (always pointing to ".")
#[macro_export]
macro_rules! cwd {
    () => {{
        use anyhow::Context;
        std::env::current_dir().context("Cannot read current directory")?
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{ConfigLevel, Repository};
    use std::{
        fs::{self},
        path::Path,
    };
    use tempdir::TempDir;

    const CONFIG_TEMPLATE: &str = r#"
    [user]
        name = Test User
        email = test@example.com
    "#;

    fn init_repo() -> Result<(TempDir, Repository)> {
        let dir = TempDir::new("git")?;
        let dir_path = dir.path();

        println!("initiating repo in {:?}", dir_path);
        let repo = Repository::init(dir_path)?;

        fs::write(dir_path.join(".gitconfig"), CONFIG_TEMPLATE)?;

        let mut config = repo.config()?;
        config.add_file(&dir_path.join(".gitconfig"), ConfigLevel::Local, true)?;

        Ok((dir, repo))
    }

    macro_rules! commit {
        ($repo:expr, $msg:expr) => {
            let mut index = $repo.index()?;
            let oid = index.write_tree()?;
            let signature = $repo.signature()?;
            let head = $repo.head();
            let parent = if head.is_ok() {
                match head?.peel_to_commit() {
                    Ok(commit) => {
                        vec![commit]
                    }
                    Err(_) => {
                        vec![]
                    }
                }
            } else {
                vec![]
            };

            $repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                $msg,
                &$repo.find_tree(oid)?,
                &parent.iter().collect::<Vec<_>>(),
            )?;
        };
    }

    macro_rules! tag {
        ($repo:expr, $name:expr) => {
            let signature = $repo.signature()?;
            let obj = $repo.revparse_single("HEAD")?;
            $repo.tag($name, &obj, &signature, "", false)?;
        };
    }

    #[test]
    fn test_get_current_tag() -> Result<(), Box<dyn std::error::Error>> {
        let (tmp, repo) = init_repo()?;
        let path = tmp.path();
        let file_path = path.join("test.txt");

        fs::write(file_path, "Hello, world!")?;

        let mut index = repo.index()?;
        index.add_path(Path::new("test.txt"))?;

        commit!(repo, "Initial commit");

        tag!(repo, "v1.0.0");

        commit!(repo, "commit");

        tag!(repo, "v2.0.0");

        let tag = get_current_tag(path)?;

        assert_eq!(tag.name(), "v2.0.0");

        Ok(())
    }

    #[test]
    fn test_get_current_tag_no_tags() -> Result<(), Box<dyn std::error::Error>> {
        let (path, _) = init_repo()?;

        let result = get_current_tag(path.path());

        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_handle_different_formats() -> Result<(), Box<dyn std::error::Error>> {
        let (path, repo) = init_repo()?;

        fs::write(path.path().join("test.txt"), "Hello, world!")?;

        let mut index = repo.index()?;
        index.add_path(Path::new("test.txt"))?;

        commit!(repo, "Initial commit");
        tag!(repo, "v1.1.1");

        commit!(repo, "Second commit");

        tag!(repo, "v1.1.9");
        println!("creating commit 3");

        commit!(repo, "Third commit");

        tag!(repo, "v1.1.10-beta");

        let tag = get_current_tag(path.path())?;

        assert_eq!(tag.name(), "v1.1.10-beta");
        Ok(())
    }
}
