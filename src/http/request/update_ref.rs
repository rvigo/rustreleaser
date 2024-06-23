use serde::Serialize;

#[derive(Serialize)]
pub struct UpdateRefRequest {
    pub sha: String,
}
