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
    pub fn archive_url(&self, compression: &Compression) -> String {
        match compression {
            Compression::TarGz => {
                format!(
                    "https://github.com/{}/{}/archive/refs/tags/{}.tar.gz",
                    self.owner, self.repo, self.name
                )
            }
            _ => self.zipball_url.to_owned(),
        }
    }

    pub async fn download_tarball(&self, compression: &Compression) -> Result<Vec<u8>> {
        let url = self.archive_url(compression);
        let tarball = github_client::instance().download_tarball(&url).await?;
        Ok(tarball)
    }
}
