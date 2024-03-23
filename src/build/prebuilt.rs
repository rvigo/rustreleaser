use super::{arch::Arch, os::Os};
use serde::{Deserialize, Serialize};
use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreBuiltAsset {
    pub path: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch: Option<Arch>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<Os>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreBuilt {
    pub prebuilt: Vec<PreBuiltAsset>,
}

impl Deref for PreBuilt {
    type Target = Vec<PreBuiltAsset>;

    fn deref(&self) -> &Self::Target {
        &self.prebuilt
    }
}

impl DerefMut for PreBuilt {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.prebuilt
    }
}

impl PreBuilt {
    pub fn is_multi_target(&self) -> bool {
        self.len() > 1 && (self.is_multi_arch() || self.is_multi_os())
    }

    pub fn is_multi_arch(&self) -> bool {
        self.iter().any(|asset| asset.arch.is_some())
    }

    pub fn is_multi_os(&self) -> bool {
        self.iter().any(|asset| asset.os.is_some())
    }
}
