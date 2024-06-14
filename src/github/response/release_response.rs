use serde::Deserialize;

#[derive(Deserialize)]
pub struct ReleaseResponse {
    pub id: u64,
    pub name: String,
    pub tarball_url: String,
    pub zipball_url: String,
}
