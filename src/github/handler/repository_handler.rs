use super::{
    branch_handler::BranchHandler, branches_handler::BranchesHandler,
    pull_request_handler::PullRequestHandler, release_handler::ReleaseHandler,
};

pub struct RepositoryHandler {
    owner: String,
    repo: String,
}

impl RepositoryHandler {
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        RepositoryHandler {
            owner: owner.into(),
            repo: repo.into(),
        }
    }

    pub fn releases(&self) -> ReleaseHandler {
        ReleaseHandler::new(&self.owner, &self.repo)
    }

    pub fn branches(&self) -> BranchesHandler {
        BranchesHandler::new(&self.owner, &self.repo)
    }

    pub fn branch(&self, branch: impl Into<String>) -> BranchHandler {
        BranchHandler::new(&self.owner, &self.repo, branch)
    }

    pub fn pull_request(&self) -> PullRequestHandler {
        PullRequestHandler::new(&self.owner, &self.repo)
    }
}
