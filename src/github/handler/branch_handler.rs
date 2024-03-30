use crate::github::{
    builder::upsert_file_builder::UpsertFileBuilder, github_client, response::sha_response::Sha,
};
use anyhow::Result;

pub struct BranchHandler {
    owner: String,
    repo: String,
    base: String,
}

impl BranchHandler {
    pub fn new(
        owner: impl Into<String>,
        repo: impl Into<String>,
        branch: impl Into<String>,
    ) -> Self {
        BranchHandler {
            owner: owner.into(),
            repo: repo.into(),
            base: branch.into(),
        }
    }

    pub fn upsert_file(&self) -> UpsertFileBuilder {
        UpsertFileBuilder::new(&self.owner, &self.repo, &self.base)
    }

    pub async fn get_commit_sha(&self) -> Result<Sha> {
        github_client::instance()
            .get_commit_sha(&self.owner, &self.repo, &self.base)
            .await
    }
}
