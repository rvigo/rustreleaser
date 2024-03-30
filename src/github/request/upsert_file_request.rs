use super::committer_request::CommitterRequest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UpsertFileRequest {
    pub message: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
    pub committer: CommitterRequest,
}

impl UpsertFileRequest {
    pub fn new(
        message: impl Into<String>,
        content: impl Into<String>,
        branch: Option<String>,
        sha: Option<String>,
        committer: CommitterRequest,
    ) -> Self {
        Self {
            message: message.into(),
            content: content.into(),
            branch,
            sha,
            committer,
        }
    }
}
