use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PullRequest {
    pub number: u64,
    pub state: String,
    pub head_sha: String,
}
