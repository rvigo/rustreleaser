use std::future::Future;

use super::{asset::UploadedAsset, generate_checksum_asset};
use crate::{
    brew::package::Package,
    git::tag::Tag,
    github::{asset::Asset, github_client},
};
use anyhow::{bail, Result};

#[derive(Debug, Clone)]
pub struct Release {
    pub owner: String,
    pub repo: String,
    pub id: u64,
    pub packages: Vec<Package>,
}

impl Release {
    pub fn new(id: u64, owner: impl Into<String>, repo: impl Into<String>) -> Self {
        Release {
            id,
            owner: owner.into(),
            repo: repo.into(),
            packages: vec![],
        }
    }

    pub async fn upload_assets(&self, assets: Vec<Asset>, tag: &Tag) -> Result<Vec<UploadedAsset>> {
        let mut uploaded = vec![];
        for asset in assets {
            let uploaded_asset = github_client::instance()
                .upload_asset(&asset, &self.owner, tag, &self.repo, self.id)
                .await?;
            log::debug!("Uploaded asset: {:#?}", uploaded_asset);
            uploaded.push(uploaded_asset);

            if let Err(err) = self.upload_checksum_asset(&asset, tag).await {
                bail!(err)
            }
        }

        Ok(uploaded)
    }

    pub fn upload_assets_raw<'a>(
        &'a self,
        assets: &'a Vec<Asset>,
        tag: &'a Tag,
    ) -> Vec<impl Future<Output = Result<UploadedAsset>> + 'a> {
        let mut uploaded = vec![];
        for asset in assets {
            let uploaded_asset = github_client::instance().upload_asset(
                &asset,
                &self.owner,
                tag,
                &self.repo,
                self.id,
            );
            uploaded.push(uploaded_asset);
        }

        uploaded
    }

    async fn upload_checksum_asset(&self, asset: &Asset, tag: &Tag) -> Result<()> {
        let checksum_asset = generate_checksum_asset(asset)?;
        let ua = github_client::instance()
            .upload_asset(&checksum_asset, &self.owner, tag, &self.repo, self.id)
            .await?;
        log::debug!("Uploaded checksum asset: {:#?}", ua);
        Ok(())
    }

    pub fn upload_checksum_asset_raw<'a>(
        &'a self,
        tag: &'a Tag,
        checksum_asset: &'a Asset,
    ) -> impl Future<Output = Result<UploadedAsset>> + 'a {
        let ua = github_client::instance().upload_asset(
            &checksum_asset,
            &self.owner,
            tag,
            &self.repo,
            self.id,
        );
        ua
    }
}
