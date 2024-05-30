use super::builder::create_pull_request_builder::CreatePullRequestBuilder;

pub struct PullRequestHandler {
    owner: String,
    repo: String,
}

impl PullRequestHandler {
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        PullRequestHandler {
            owner: owner.into(),
            repo: repo.into(),
        }
    }

    pub fn create(&self) -> CreatePullRequestBuilder {
        CreatePullRequestBuilder::new(&self.owner, &self.repo)
    }
}
