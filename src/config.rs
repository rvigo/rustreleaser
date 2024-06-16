use crate::{
    brew::{dependency::Symbol, repository::Repository},
    compression::Compression,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};

const MAIN_BRANCH_NAME: &str = "main";
const BREW_DEFAULT_COMMIT_MESSAGE: &str = "update formula";

const PR_DEFAULT_BASE_BRANCH_NAME: &str = MAIN_BRANCH_NAME;
const PR_DEFAULT_HEAD_BRANCH_NAME: &str = "bumps-formula-version";

const DEFAULT_CONFIG_FILE_NAME: &str = "rustreleaser.yaml";

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub brew: BrewConfig,
    pub release: ReleaseConfig,
}

impl Config {
    pub async fn load() -> Result<Config> {
        let config_string = tokio::fs::read_to_string(DEFAULT_CONFIG_FILE_NAME).await?;

        let config = serde_yaml::from_str::<Config>(&config_string)?;

        Ok(config)
    }
}

#[derive(Serialize, Deserialize)]
pub struct BrewConfig {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub homepage: String,
    pub install: String,
    #[serde(default)]
    pub license: String,
    pub head: Option<Head>,
    #[serde(default)]
    pub test: String,
    #[serde(default)]
    pub caveats: String,
    #[serde(default = "BrewConfig::default_commit_message")]
    pub commit_message: String,
    pub commit_author: Option<CommitterConfig>,
    pub pull_request: Option<PullRequestConfig>,
    pub repository: Repository,
    pub dependencies: Vec<DependsOnConfig>,
    pub with_version: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Head {
    pub url: String,
    pub branch: String,
}

impl BrewConfig {
    fn default_commit_message() -> String {
        BREW_DEFAULT_COMMIT_MESSAGE.to_owned()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitterConfig {
    pub email: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PullRequestConfig {
    pub title: Option<String>,
    pub body: Option<String>,
    pub labels: Option<Vec<String>>,
    pub assignees: Option<Vec<String>>,
    #[serde(default)]
    pub draft: bool,
    #[serde(default = "PullRequestConfig::default_base_branch_name")]
    pub base: String,
    #[serde(default = "PullRequestConfig::default_head_branch_name")]
    pub head: String,
}

impl Default for PullRequestConfig {
    fn default() -> Self {
        PullRequestConfig {
            title: Some("Bump formula version".to_owned()),
            body: None,
            labels: None,
            assignees: None,
            draft: false,
            base: PullRequestConfig::default_base_branch_name(),
            head: PullRequestConfig::default_head_branch_name(),
        }
    }
}

impl PullRequestConfig {
    fn default_base_branch_name() -> String {
        PR_DEFAULT_BASE_BRANCH_NAME.to_owned()
    }

    fn default_head_branch_name() -> String {
        PR_DEFAULT_HEAD_BRANCH_NAME.to_owned()
    }
}

#[derive(Serialize, Deserialize)]
pub struct ReleaseConfig {
    pub owner: String,
    pub repo: String,
    #[serde(default = "ReleaseConfig::target_branch")]
    pub target_branch: String,
    #[serde(default)]
    pub prerelease: bool,
    #[serde(default)]
    pub draft: bool,
    pub name: Option<String>,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub compression: Compression,
}

impl ReleaseConfig {
    pub fn target_branch() -> String {
        MAIN_BRANCH_NAME.to_owned()
    }
}

#[derive(Serialize, Deserialize)]
pub struct DependsOnConfig {
    pub name: String,
    pub symbol: OneOrMany<Symbol>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> From<OneOrMany<T>> for Vec<T> {
    fn from(from: OneOrMany<T>) -> Self {
        match from {
            OneOrMany::One(val) => vec![val],
            OneOrMany::Many(vec) => vec,
        }
    }
}
