use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct CommitShaResponse {
    pub sha: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct FileShaResponse {
    pub sha: String,
    pub name: String,
    pub path: String,
}
