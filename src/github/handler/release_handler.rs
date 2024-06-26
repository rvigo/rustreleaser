use crate::{
    git::tag::Tag,
    github::{
        github_client, handler::builder::create_release_builder::CreateReleaseBuilder,
        release::Release,
    },
};
use anyhow::Result;

pub struct ReleaseHandler {
    owner: String,
    repo: String,
}

impl ReleaseHandler {
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        ReleaseHandler {
            owner: owner.into(),
            repo: repo.into(),
        }
    }

    pub fn create(&self) -> CreateReleaseBuilder {
        CreateReleaseBuilder::new(&self.owner, &self.repo)
    }

    pub async fn get_by_tag(&self, tag: &Tag) -> Result<Release> {
        github_client::instance()
            .get_release_by_tag(&self.owner, &self.repo, tag)
            .await
    }
}
