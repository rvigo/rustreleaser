use crate::build::{arch::Arch, os::Os};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub os: Option<Os>,
    pub arch: Option<Arch>,
    pub url: String,
    pub sha256: String,
    pub prebuilt: bool,
}

impl Package {
    pub fn new(
        name: impl Into<String>,
        os: Option<Os>,
        arch: Option<Arch>,
        url: impl Into<String>,
        sha256: impl Into<String>,
        prebuilt: bool,
    ) -> Self {
        Self {
            name: name.into(),
            os,
            arch,
            url: url.into(),
            sha256: sha256.into(),
            prebuilt,
        }
    }
}
