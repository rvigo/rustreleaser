use serde::Deserialize;

#[derive(Deserialize)]
pub struct Object {
    pub sha: String,
    pub r#type: String,
}
