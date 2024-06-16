use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BranchRefRequest {
    pub r#ref: String,
    pub sha: String,
}

impl BranchRefRequest {
    pub fn new(r#ref: impl Into<String>, sha: impl Into<String>) -> Self {
        Self {
            r#ref: format!("refs/heads/{}", r#ref.into()),
            sha: sha.into(),
        }
    }
}
