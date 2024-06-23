use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct BlobResponse {
    pub sha: String,
}
