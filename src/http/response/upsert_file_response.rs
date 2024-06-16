#[derive(Debug, serde::Deserialize)]
pub struct UpsertFileResponse {
    pub content: Content,
}

#[derive(Debug, serde::Deserialize)]
pub struct Content {
    pub name: String,
    pub path: String,
    pub sha: String,
}
