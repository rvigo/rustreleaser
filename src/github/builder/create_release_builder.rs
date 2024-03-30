use super::BuilderExecutor;
use crate::github::{dto::release_dto::ReleaseDto, github_client, release::Release, tag::Tag};
use anyhow::Result;

pub struct CreateReleaseBuilder {
    pub owner: String,
    pub repo: String,
    pub release_name: String,
    pub release_tag: Tag,
    pub target_branch: String,
    pub draft: Option<bool>,
    pub prerelease: Option<bool>,
    pub body: Option<String>,
}

impl CreateReleaseBuilder {
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        CreateReleaseBuilder {
            owner: owner.into(),
            repo: repo.into(),
            release_name: String::new(),
            release_tag: Tag::empty(),
            target_branch: String::new(),
            draft: None,
            prerelease: None,
            body: None,
        }
    }

    pub fn name(mut self, release_name: impl Into<String>) -> Self {
        self.release_name = release_name.into();
        self
    }

    pub fn tag(mut self, release_tag: &Tag) -> Self {
        self.release_tag = release_tag.to_owned();
        self
    }

    pub fn target_branch(mut self, target_branch: impl Into<String>) -> Self {
        self.target_branch = target_branch.into();
        self
    }

    pub fn draft(mut self, draft: bool) -> Self {
        self.draft = Some(draft);
        self
    }

    pub fn prerelease(mut self, pre_release: bool) -> Self {
        self.prerelease = Some(pre_release);
        self
    }

    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }
}

impl BuilderExecutor for CreateReleaseBuilder {
    type Output = Release;

    async fn execute(self) -> Result<Release> {
        let release = ReleaseDto::new(
            self.owner,
            self.repo,
            self.release_tag,
            self.target_branch,
            self.release_name,
            self.draft.unwrap(),
            self.prerelease.unwrap(),
            self.body.unwrap_or_default(),
        );
        github_client::instance().create_release(release).await
    }
}
