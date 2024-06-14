use crate::{compression::Compression, github::github_client};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Release {
    pub id: u64,
    pub owner: String,
    pub repo: String,
    pub name: String,
    pub tarball_url: String,
    pub zipball_url: String,
}

impl Release {
    pub fn new(
        id: u64,
        owner: impl Into<String>,
        repo: impl Into<String>,
        name: impl Into<String>,
        tarball_url: impl Into<String>,
        zipball_url: impl Into<String>,
    ) -> Self {
        Release {
            id,
            owner: owner.into(),
            repo: repo.into(),
            name: name.into(),
            tarball_url: tarball_url.into(),
            zipball_url: zipball_url.into(),
        }
    }

    pub async fn download_tarball(&self, compression: &Compression) -> Result<Vec<u8>> {
        let tarball = github_client::instance()
            .download_tarball(&self.tarball_url(compression))
            .await?;
        Ok(tarball)
    }

    fn tarball_url(&self, compression: &Compression) -> String {
        match compression {
            Compression::TarGz => self.tarball_url.to_owned(),
            _ => self.zipball_url.to_owned(),
        }
    }

    pub fn tarball_name(&self, compression: &Compression) -> String {
        let url = self.tarball_url(compression);
        let url_parts: Vec<&str> = url.split('/').collect();
        url_parts.last().unwrap().to_owned().to_owned()
    }
}
