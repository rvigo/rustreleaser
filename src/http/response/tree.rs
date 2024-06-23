use serde::Deserialize;

#[derive(Deserialize)]
pub struct TreeResponse {
    pub sha: String,
}
