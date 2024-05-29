use super::BuilderExecutor;
use crate::{
    git::committer::Committer,
    github::{dto::commit_info_dto::CommitInfoDto, github_client},
};

pub struct UpsertFileBuilder {
    owner: String,
    repo: String,
    path: String,
    commit_message: String,
    content: String,
    committer: Committer,
    head: String,
}

impl UpsertFileBuilder {
    pub fn new(
        owner: impl Into<String>,
        repo: impl Into<String>,
        branch: impl Into<String>,
    ) -> Self {
        UpsertFileBuilder {
            owner: owner.into(),
            repo: repo.into(),
            path: String::new(),
            commit_message: String::new(),
            content: String::new(),
            committer: Committer::default(),
            head: branch.into(),
        }
    }

    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.commit_message = message.into();
        self
    }

    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    pub fn committer(mut self, committer: &Committer) -> Self {
        self.committer = committer.to_owned();
        self
    }
}

impl BuilderExecutor for UpsertFileBuilder {
    type Output = ();

    async fn execute(self) -> anyhow::Result<Self::Output> {
        let commit_info = CommitInfoDto::new(&self.commit_message, &self.committer);

        github_client::instance()
            .upsert_file(
                &self.owner,
                &self.repo,
                &self.path,
                &self.content,
                self.head,
                commit_info,
            )
            .await
    }
}
