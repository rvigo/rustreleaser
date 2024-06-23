use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BranchRefRequest {
    pub r#ref: String,
    pub sha: String,
}

impl BranchRefRequest {
    pub fn new(r#ref: impl Into<String>, sha: impl Into<String>) -> Self {
        let r#ref: String = r#ref.into();
        let sha: String = sha.into();

        Self {
            r#ref: format!("refs/heads/{}", r#ref),
            sha,
        }
    }
}
