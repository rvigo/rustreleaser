use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateReleaseRequest {
    pub tag_name: String,
    pub target_commitish: String,
    pub name: String,
    pub body: String,
    pub draft: bool,
    pub prerelease: bool,
}

impl CreateReleaseRequest {
    pub fn new(
        tag_name: impl Into<String>,
        target_commitish: impl Into<String>,
        name: impl Into<String>,
        body: impl Into<String>,
        draft: bool,
        prerelease: bool,
    ) -> Self {
        Self {
            tag_name: tag_name.into(),
            target_commitish: target_commitish.into(),
            name: name.into(),
            body: body.into(),
            draft,
            prerelease,
        }
    }
}
