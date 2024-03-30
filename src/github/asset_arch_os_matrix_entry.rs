use super::asset::Asset;
use crate::{
    arch_os_matrix::Entry,
    build::{arch::Arch, compression::Compression, os::Os},
};

#[derive(Debug, Clone)]
pub struct AssetArchOsMatrixEntry<'matrix> {
    pub arch: &'matrix Arch,
    pub os: &'matrix Os,
    pub name: String,
    pub asset: Option<Asset>,
}

impl<'matrix> AssetArchOsMatrixEntry<'matrix> {
    pub fn new(
        arch: &'matrix Arch,
        os: &'matrix Os,
        name: impl Into<String>,
        tag: &'matrix str,
        compression: &'matrix Compression,
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
        }
    }

    pub fn set_asset(&mut self, asset: Asset) {
        self.asset = Some(asset);
    }
}

impl Entry for AssetArchOsMatrixEntry<'_> {}
