use super::committer_request::CommitterRequest;
use crate::git::committer::Committer;
use serde::Serialize;

#[derive(Serialize)]
pub struct CommitRequest {
    pub message: String,
    pub tree: String,
    pub parents: Vec<String>,
    pub committer: CommitterRequest,
}

impl CommitRequest {
    pub fn new(message: impl Into<String>, tree: impl Into<String>, parents: Vec<String>) -> Self {
        CommitRequest {
            message: message.into(),
            tree: tree.into(),
            parents,
            committer: Committer::default().into(),
        }
    }
}
