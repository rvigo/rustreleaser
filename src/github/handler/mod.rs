pub mod branch_handler;
pub mod branches_handler;
mod builder;
pub mod pull_request_handler;
pub mod release_handler;
pub mod repository_handler;

use super::github_client::GithubClient;
pub use builder::BuilderExecutor;
use repository_handler::RepositoryHandler;

/// Github repo handler access implementation
impl GithubClient {
    pub fn repo(&self, owner: impl Into<String>, name: impl Into<String>) -> RepositoryHandler {
        RepositoryHandler::new(owner, name)
    }
}
