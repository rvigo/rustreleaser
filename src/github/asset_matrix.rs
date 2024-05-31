use super::asset::{Asset, UploadedAsset};
use crate::{
    brew::package::Package,
    build::{arch::Arch, os::Os},
    compression::Compression,
};
use anyhow::Context;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone)]
pub struct AssetMatrixEntry {
    pub arch: Arch,
    pub os: Os,
    pub name: String,
    pub asset: Option<Asset>,
    pub prebuilt: bool,
}

impl AssetMatrixEntry {
    pub fn new(
        arch: Arch,
        os: Os,
        name: impl Into<String>,
        tag: impl Into<String>,
        compression: Compression,
        prebuilt: bool,
    ) -> Self {
        let name = format!(
            "{}_{}_{}_{}.{}",
            name.into(),
            tag.into(),
            arch.to_string(),
            os.to_string(),
            compression.extension()
        );
        Self {
            arch,
            os,
            name,
            asset: None,
            prebuilt,
        }
    }

    pub fn set_asset(&mut self, asset: Asset) {
        self.asset = Some(asset);
    }
}

#[derive(Clone)]
pub struct EnrichedMatrixEntry {
    entry: AssetMatrixEntry,
    uploaded_asset: UploadedAsset,
}

impl EnrichedMatrixEntry {
    pub fn new(entry: AssetMatrixEntry, uploaded_asset: UploadedAsset) -> Self {
        Self {
            entry,
            uploaded_asset,
        }
    }

    pub fn into_package(self) -> Package {
        Package::new(
            self.uploaded_asset.name.to_owned(),
            Some(self.entry.os.to_owned()),
            Some(self.entry.arch.to_owned()),
            self.uploaded_asset.url.to_owned(),
            self.uploaded_asset.checksum.to_owned(),
            self.entry.prebuilt,
        )
    }
}

#[derive(Default)]
pub struct AssetMatrix(Vec<AssetMatrixEntry>);

impl AssetMatrix {
    pub fn enrich(&self, uploaded_assets: Vec<UploadedAsset>) -> Vec<EnrichedMatrixEntry> {
        self.iter()
            .map(|entry| {
                let uploaded_asset = uploaded_assets
                    .iter()
                    .find(|asset| asset.name == entry.name)
                    .context(format!("Asset {} not found", entry.name))
                    .expect("Asset not found");

                EnrichedMatrixEntry::new(entry.to_owned(), uploaded_asset.to_owned())
            })
            .collect()
    }
}

impl Deref for AssetMatrix {
    type Target = Vec<AssetMatrixEntry>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AssetMatrix {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
