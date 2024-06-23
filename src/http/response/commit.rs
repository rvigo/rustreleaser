use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct CommitResponse {
    pub sha: String,
}
