pub mod package;
pub mod repository;
pub mod target;
mod template;

use self::{
    package::Package,
    repository::Repository,
    target::{MultiTarget, SingleTarget, Target, Targets},
};
use crate::{
    brew::template::handlebars,
    cwd,
    git::{committer::Committer, tag::Tag},
    github::{github_client, handler::BuilderExecutor},
};
use crate::{
    build::arch::Arch,
    config::{BrewConfig, CommitterConfig, PullRequestConfig},
    git,
};
use anyhow::{Context, Result};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fs;
use template::Template;

#[derive(Debug, Serialize, Deserialize)]
pub struct Brew {
    pub name: String,
    pub description: String,
    pub homepage: String,
    pub license: String,
    pub head: String,
    pub test: String,
    pub caveats: String,
    pub commit_message: String,
    pub commit_author: Option<CommitterConfig>,
    pub install_info: String,
    pub repository: Repository,
    #[serde(flatten)]
    #[serde(rename(serialize = "version"))]
    pub tag: Tag,
    pub pull_request: Option<PullRequestConfig>,
    pub targets: Targets,
    pub template: Template,
}

impl Brew {
    pub fn new(brew: BrewConfig, version: Tag, packages: Vec<Package>) -> Brew {
        let targets = Targets::from(packages);
        let template = Template::from(&targets);
        Brew {
            name: captalize(brew.name),
            description: brew.description,
            homepage: brew.homepage,
            install_info: brew.install,
            repository: brew.repository,
            tag: version,
            targets,
            license: brew.license,
            head: brew.head,
            test: brew.test,
            caveats: brew.caveats,
            commit_message: brew.commit_message,
            commit_author: brew.commit_author,
            pull_request: brew.pull_request,
            template,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BrewArch {
    pub arch: Arch,
    pub url: String,
    pub hash: String,
}

impl BrewArch {
    pub fn new(arch: Arch, url: impl Into<String>, hash: impl Into<String>) -> Self {
        Self {
            arch,
            url: url.into(),
            hash: hash.into(),
        }
    }
}

pub async fn publish(brew_config: BrewConfig, packages: Vec<Package>) -> Result<String> {
    log::debug!("packages: {:?}", packages);

    let brew = Brew::new(brew_config, git::get_current_tag(cwd!())?, packages);
    log::debug!("Rendering Formula template {}", brew.template.to_string());

    let data = serialize(&brew)?;

    write_file(format!("{}.rb", brew.name.to_lowercase()), &data)?;

    if brew.pull_request.is_some() {
        log::debug!("Creating pull request");
        push_formula(brew).await?;
    } else {
        log::debug!("Committing file to head branch");
        github_client::instance()
            .repo(&brew.repository.owner, &brew.repository.name)
            .branch(&brew.head)
            .upsert_file()
            .path(format!("{}.rb", brew.name.to_lowercase()))
            .message(brew.commit_message)
            .content(&data)
            .execute()
            .await
            .context("error uploading file to main branch")?;
    }

    Ok(data)
}

fn serialize(brew: &Brew) -> Result<String> {
    let hb = handlebars()?;
    let rendered = hb.render(&brew.template.to_string(), brew)?;
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
    let pull_request = brew.pull_request.unwrap();

    let committer = brew.commit_author.map(Committer::from).unwrap_or_default();
    let repo = github_client::instance().repo(&brew.repository.owner, &brew.repository.name);

    log::debug!("Creating branch");
    let sha = repo
        .branch(&pull_request.base)
        .get_commit_sha()
        .await
        .context("error getting the base branch commit sha")?;

    repo.branches()
        .create()
        .branch(&pull_request.head)
        .sha(sha.sha)
        .execute()
        .await
        .context("error creating the branch")?;

    let content = fs::read_to_string(format!("{}.rb", brew.name))?;

    log::debug!("Updating formula");
    repo.branch(&pull_request.head)
        .upsert_file()
        .path(format!("{}.rb", brew.name))
        .message(brew.commit_message)
        .content(content)
        .committer(&committer)
        .execute()
        .await
        .context("error uploading file to head branch")?;

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
        .context("error creating pull request")?;

    Ok(())
}

impl From<Vec<Package>> for Targets {
    fn from(value: Vec<Package>) -> Targets {
        let targets: Vec<Target> = if value.is_empty() {
            vec![]
        } else if value.len() == 1 {
            let target = vec![Target::Single(SingleTarget::new(
                &value[0].url,
                &value[0].sha256,
            ))];
            target
        } else {
            let group = value
                .iter()
                .cloned()
                .group_by(|p| p.os.to_owned())
                .into_iter()
                .map(|g| MultiTarget {
                    os: g.0.unwrap(),
                    archs: g
                        .1
                        .map(|p| BrewArch::new(p.arch.unwrap(), p.url, p.sha256))
                        .collect(),
                })
                .map(Target::Multi)
                .collect();

            group
        };

        Targets(targets)
    }
}

impl From<CommitterConfig> for Committer {
    fn from(value: CommitterConfig) -> Self {
        Committer {
            author: value.name,
            email: value.email,
        }
    }
}
