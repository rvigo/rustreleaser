use super::asset::{Asset, UploadedAsset};
use crate::{
    brew::package::Package,
    build::{arch::Arch, os::Os},
    compression::Compression,
};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone)]
pub struct AssetArchOsMatrixEntry<'matrix> {
    pub arch: &'matrix Arch,
    pub os: &'matrix Os,
    pub name: String,
    pub asset: Option<Asset>,
    pub prebuilt: bool,
}

impl<'matrix> AssetArchOsMatrixEntry<'matrix> {
    pub fn new(
        arch: &'matrix Arch,
        os: &'matrix Os,
        name: impl Into<String>,
        tag: &'matrix str,
        compression: &'matrix Compression,
        prebuilt: bool,
    ) -> Self {
        let name = format!(
            "{}_{}_{}_{}.{}",
            name.into(),
            tag,
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
pub struct EnrichedAssetArchOsMatrixEntry<'matrix> {
    entry: AssetArchOsMatrixEntry<'matrix>,
    uploaded_asset: UploadedAsset,
}

impl<'a> EnrichedAssetArchOsMatrixEntry<'a> {
    pub fn new(entry: AssetArchOsMatrixEntry<'a>, uploaded_asset: UploadedAsset) -> Self {
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
pub struct AssetArchOsMatrix<'matrix>(Vec<AssetArchOsMatrixEntry<'matrix>>);

impl AssetArchOsMatrix<'_> {
    pub fn enrich(
        &self,
        uploaded_assets: Vec<UploadedAsset>,
    ) -> Vec<EnrichedAssetArchOsMatrixEntry> {
        self.iter()
            .map(|entry| {
                let uploaded_asset = uploaded_assets
                    .iter()
                    .find(|asset| asset.name == entry.name)
                    .expect("asset not found");

                EnrichedAssetArchOsMatrixEntry::new(entry.to_owned(), uploaded_asset.to_owned())
            })
            .collect()
    }
}

impl<'deref> Deref for AssetArchOsMatrix<'deref> {
    type Target = Vec<AssetArchOsMatrixEntry<'deref>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AssetArchOsMatrix<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
