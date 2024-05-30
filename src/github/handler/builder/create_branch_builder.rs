use super::BuilderExecutor;
use crate::github::github_client;
use anyhow::Result;

pub struct CreateBranchBuilder {
    owner: String,
    repo: String,
    branch: String,
    sha: String,
}

impl CreateBranchBuilder {
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        CreateBranchBuilder {
            owner: owner.into(),
            repo: repo.into(),
            branch: String::new(),
            sha: String::new(),
        }
    }

    pub fn branch(mut self, branch: impl Into<String>) -> Self {
        self.branch = branch.into();
        self
    }

    pub fn sha(mut self, sha: impl Into<String>) -> Self {
        self.sha = sha.into();
        self
    }
}

impl BuilderExecutor for CreateBranchBuilder {
    type Output = ();

    async fn execute(self) -> Result<Self::Output> {
        github_client::instance()
            .create_branch(&self.owner, &self.repo, &self.branch, &self.sha)
            .await
    }
}
