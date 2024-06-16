mod dto;
pub mod github_client;
pub mod handler;
mod release;

use self::release::Release;
use crate::{
    checksum::Checksum,
    config::ReleaseConfig,
    cwd,
    git::{self, tag::Tag},
};
use anyhow::Result;
use handler::BuilderExecutor;
use std::io::Cursor;

pub struct ReleaseOutput {
    pub checksum: Checksum,
    pub tag_name: String,
    pub tarball_url: String,
}

pub async fn release(release_config: &ReleaseConfig) -> Result<ReleaseOutput> {
    let tag = git::get_current_tag(cwd!())?;

    log::debug!("getting/creating release");
    let release = get_release(release_config, &tag).await?;

    let tarball = release
        .download_tarball(&release_config.compression)
        .await?;
    log::debug!("generating checksum");
    let checksum = Checksum::create(Cursor::new(tarball))?;

    let dto = ReleaseOutput {
        checksum,
        tag_name: release.tag_name.to_owned(),
        tarball_url: release.archive_url(&release_config.compression),
    };
    Ok(dto)
}

async fn get_release(release_config: &ReleaseConfig, tag: &Tag) -> Result<Release> {
    let res = get_release_by_tag(release_config, tag).await;
    match res {
        Ok(release) => {
            log::info!("found release by tag: {:?}", tag);
            Ok(release)
        }
        Err(err) => {
            log::warn!(
                "cannot find a release by tag: {:?}, trying to create a new one",
                err
            );

            create_release(release_config, tag).await
        }
    }
}

async fn get_release_by_tag(release_config: &ReleaseConfig, tag: &Tag) -> Result<Release> {
    github_client::instance()
        .repo(&release_config.owner, &release_config.repo)
        .releases()
        .get_by_tag(tag)
        .await
}

async fn create_release(release_config: &ReleaseConfig, tag: &Tag) -> Result<Release> {
    github_client::instance()
        .repo(&release_config.owner, &release_config.repo)
        .releases()
        .create()
        .tag(tag)
        .target_branch(&release_config.target_branch)
        .name(
            release_config
                .name
                .as_ref()
                .unwrap_or(&tag.name().to_owned()),
        )
        .draft(release_config.draft)
        .prerelease(release_config.prerelease)
        .body(&release_config.body)
        .execute()
        .await
}
