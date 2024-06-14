pub mod repository;
mod template;

use self::repository::Repository;
use crate::{
    brew::template::handlebars,
    checksum::Checksum,
    config::Head,
    cwd,
    git::{committer::Committer, tag::Tag},
    github::{github_client, handler::BuilderExecutor, ReleaseDto},
};
use crate::{
    config::{BrewConfig, CommitterConfig, PullRequestConfig},
    git,
};
use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use template::FORMULA_FILE_TEMPLATE;

#[derive(Serialize)]
pub struct Brew {
    pub name: String,
    pub description: String,
    pub homepage: String,
    pub license: String,
    pub head: Option<Head>,
    pub test: String,
    pub caveats: String,
    pub commit_message: String,
    pub commit_author: Option<CommitterConfig>,
    pub install_info: String,
    pub repository: Repository,
    pub version: String,
    pub pull_request: Option<PullRequestConfig>,
    #[serde(serialize_with = "serialize_checksum")]
    pub tarball_checksum: Checksum,
    pub url: String,
}

impl Brew {
    pub fn new(brew: BrewConfig, tag: Tag, checksum: Checksum, tarball_url: String) -> Brew {
        Brew {
            name: captalize(brew.name),
            description: captalize(brew.description),
            homepage: brew.homepage,
            install_info: brew.install.trim().to_owned(),
            repository: brew.repository,
            version: tag.strip_v_prefix().to_owned(),
            license: brew.license,
            head: brew.head,
            test: brew.test,
            caveats: brew.caveats,
            commit_message: brew.commit_message,
            commit_author: brew.commit_author,
            pull_request: brew.pull_request,
            tarball_checksum: checksum,
            url: tarball_url,
        }
    }
}

fn serialize_checksum<S>(checksum: &Checksum, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(checksum.value())
}

pub async fn publish(brew_config: BrewConfig, dto: ReleaseDto) -> Result<String> {
    let brew = Brew::new(
        brew_config,
        git::get_current_tag(cwd!())?,
        dto.checksum,
        dto.tarball_url,
    );

    let data = serialize(&brew).context("Cannot serialize the formula file")?;

    write_file(format!("{}.rb", brew.name.to_lowercase()), &data)
        .context("Cannot write to the formula file")?;

    log::debug!("Creating pull request");
    push_formula(brew)
        .await
        .context("Cannot create the pull request for the formula")?;
    Ok(data)
}

fn serialize(brew: &Brew) -> Result<String> {
    let hb = handlebars()?;
    let rendered = hb.render(FORMULA_FILE_TEMPLATE, brew)?;
    Ok(rendered)
}

fn write_file(file_name: impl Into<String>, data: impl Into<String>) -> Result<()> {
    fs::write(file_name.into(), data.into())?;
    Ok(())
}

fn captalize(mut string: String) -> String {
    format!("{}{string}", string.remove(0).to_uppercase())
}

async fn push_formula(brew: Brew) -> Result<()> {
    let pull_request = brew.pull_request.unwrap_or_default();

    let committer = brew.commit_author.map(Committer::from).unwrap_or_default();
    let repo = github_client::instance().repo(&brew.repository.owner, &brew.repository.name);

    log::debug!("Creating branch");
    let sha = repo
        .branch(&pull_request.base)
        .get_commit_sha()
        .await
        .context("Error getting the base branch commit sha")?;

    repo.branches()
        .create()
        .branch(&pull_request.head)
        .sha(sha.sha)
        .execute()
        .await
        .context("Error creating the branch")?;

    let formula_name = format!("{}.rb", brew.name.to_lowercase());

    let content = fs::read_to_string(&formula_name).context(format!(
        "Cannot read the rb file with name {}",
        formula_name
    ))?;

    log::debug!("Updating formula");
    repo.branch(&pull_request.head)
        .upsert_file()
        .path(formula_name)
        .message(brew.commit_message)
        .content(content)
        .committer(&committer)
        .execute()
        .await
        .context("Error uploading file to head branch")?;

    log::debug!("Creating pull request");
    repo.pull_request()
        .create()
        .assignees(pull_request.assignees.unwrap_or_default())
        .base(pull_request.base)
        .head(&pull_request.head)
        .body(pull_request.body.unwrap_or_default())
        .labels(pull_request.labels.unwrap_or_default())
        .title(pull_request.title.unwrap_or_default())
        .committer(&committer)
        .execute()
        .await
        .context("Error creating pull request")?;

    Ok(())
}

impl From<CommitterConfig> for Committer {
    fn from(value: CommitterConfig) -> Self {
        Committer {
            author: value.name,
            email: value.email,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::serialize;
    use crate::{
        brew::{repository::Repository, Brew},
        checksum::Checksum,
        config::BrewConfig,
        git::tag::Tag,
    };
    use tempdir::TempDir;

    #[test]
    fn should_serialize_tag_without_v() {
        let dir = TempDir::new("brew").unwrap();
        let tag = Tag::new("v1.0.0");

        let brew_config = BrewConfig {
            name: "test".to_owned(),
            description: "test".to_owned(),
            homepage: "test".to_owned(),
            install: "test\n".to_owned(),
            repository: Repository {
                owner: "test".to_owned(),
                name: "test".to_owned(),
            },
            license: "test".to_owned(),
            head: None,
            test: "test".to_owned(),
            caveats: "test".to_owned(),
            commit_message: "test".to_owned(),
            commit_author: None,
            pull_request: None,
        };

        let brew = Brew::new(
            brew_config,
            tag,
            Checksum::create(vec![], dir.path().join("test.txt")).unwrap(),
            "url.com".to_owned(),
        );

        let serialized = serialize(&brew);

        assert!(serialized.is_ok());
    }
}
