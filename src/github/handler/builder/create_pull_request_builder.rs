use super::BuilderExecutor;
use crate::{
    git::committer::Committer,
    github::{
        dto::pull_request_dto::PullRequestDto, github_client,
        response::pull_request_response::PullRequest,
    },
};

pub struct CreatePullRequestBuilder {
    pub owner: String,
    pub repo: String,
    pub title: String,
    pub body: Option<String>,
    pub labels: Option<Vec<String>>,
    pub assignees: Option<Vec<String>>,
    pub committer: Option<Committer>,
    pub base: String,
    pub head: String,
}

impl CreatePullRequestBuilder {
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        CreatePullRequestBuilder {
            owner: owner.into(),
            repo: repo.into(),
            title: String::new(),
            body: None,
            labels: None,
            assignees: None,
            committer: None,
            base: String::new(),
            head: String::new(),
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn labels(mut self, labels: Vec<String>) -> Self {
        self.labels = Some(labels);
        self
    }

    pub fn assignees(mut self, assignees: Vec<String>) -> Self {
        self.assignees = Some(assignees);
        self
    }

    pub fn committer(mut self, committer: &Committer) -> Self {
        self.committer = Some(committer.to_owned());
        self
    }

    pub fn base(mut self, base: impl Into<String>) -> Self {
        self.base = base.into();
        self
    }

    pub fn head(mut self, head: impl Into<String>) -> Self {
        self.head = head.into();
        self
    }
}

impl BuilderExecutor for CreatePullRequestBuilder {
    type Output = PullRequest;

    async fn execute(self) -> anyhow::Result<Self::Output> {
        let pr = PullRequestDto::new(
            self.owner,
            self.repo,
            self.title,
            self.head,
            self.base,
            self.body.unwrap_or_default(),
            self.assignees.unwrap_or_default(),
            self.labels.unwrap_or_default(),
        );

        github_client::instance().create_pull_request(pr).await
    }
}
