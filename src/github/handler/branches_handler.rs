use crate::github::builder::create_branch_builder::CreateBranchBuilder;

pub struct BranchesHandler {
    owner: String,
    repo: String,
}

impl BranchesHandler {
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        BranchesHandler {
            owner: owner.into(),
            repo: repo.into(),
        }
    }

    pub fn create(&self) -> CreateBranchBuilder {
        CreateBranchBuilder::new(&self.owner, &self.repo)
    }
}
