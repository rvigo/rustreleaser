use serde::Serialize;

use crate::git::committer::Committer;

use super::committer_request::CommitterRequest;

#[derive(Serialize)]
pub struct CommitRequest {
    pub message: String,
    pub tree: String,
    pub parents: Vec<String>,
    pub commiter: CommitterRequest,
}

impl CommitRequest {
    pub fn new(message: impl Into<String>, tree: impl Into<String>, parents: Vec<String>) -> Self {
        CommitRequest {
            message: message.into(),
            tree: tree.into(),
            parents,
            commiter: Committer::default().into(),
        }
    }
}
