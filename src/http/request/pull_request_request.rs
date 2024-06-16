use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PullRequestRequest {
    pub title: String,
    pub head: String,
    pub base: String,
    pub body: String,
}

impl PullRequestRequest {
    pub fn new(
        title: impl Into<String>,
        head: impl Into<String>,
        base: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            head: head.into(),
            base: base.into(),
            body: body.into(),
        }
    }
}
