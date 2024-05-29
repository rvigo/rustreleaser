pub mod arch;
pub mod os;
pub mod prebuilt;

use self::prebuilt::PreBuiltAsset;
use arch::Arch;
use os::Os;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Build {
    pub arch: Option<Vec<Arch>>,
    pub os: Option<Vec<Os>>,
    pub binary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prebuilt: Option<Vec<PreBuiltAsset>>,
}

#[derive(PartialEq, PartialOrd)]
pub enum TargetType {
    Multi,
    Single,
    PreBuilt,
}

impl Build {
    pub fn target_type(&self) -> TargetType {
        if self.has_prebuilt() {
            TargetType::PreBuilt
        } else if self.is_multi_target() {
            TargetType::Multi
        } else {
            TargetType::Single
        }
    }

    fn is_multi_target(&self) -> bool {
        self.is_multi_arch() || self.is_multi_os()
    }

    fn is_multi_arch(&self) -> bool {
        if let Some(archs) = &self.arch {
            !archs.is_empty()
        } else {
            false
        }
    }

    fn is_multi_os(&self) -> bool {
        if let Some(oss) = &self.os {
            !oss.is_empty()
        } else {
            false
        }
    }

    fn has_prebuilt(&self) -> bool {
        self.prebuilt.is_some() && !self.prebuilt.as_ref().unwrap().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{arch::Arch, os::Os};
    use crate::build::Build;

    #[test]
    fn should_validate_if_multi_target() {
        let build = Build {
            binary: "binary".to_string(),
            arch: Some(vec![Arch::Amd64]),
            os: Some(vec![Os::UnknownLinuxGnu]),
            prebuilt: None,
        };

        assert!(build.is_multi_target());
    }

    #[test]
    fn should_validate_id_single_target() {
        let build = Build {
            binary: "binary".to_string(),
            arch: None,
            os: None,
            prebuilt: None,
        };

        assert!(!build.is_multi_target());
    }

    #[test]
    fn should_validate_if_multi_arch() {
        let build = Build {
            binary: "binary".to_string(),
            arch: Some(vec![Arch::Amd64]),
            os: None,
            prebuilt: None,
        };

        assert!(build.is_multi_arch());
    }

    #[test]
    fn should_validate_if_single_arch() {
        let build = Build {
            binary: "binary".to_string(),
            arch: None,
            os: None,
            prebuilt: None,
        };

        assert!(!build.is_multi_arch());
    }

    #[test]
    fn should_validate_if_multi_os() {
        let build = Build {
            binary: "binary".to_string(),
            arch: None,
            os: Some(vec![Os::UnknownLinuxGnu]),
            prebuilt: None,
        };

        assert!(build.is_multi_os());
    }

    #[test]
    fn should_validate_if_single_os() {
        let build = Build {
            binary: "binary".to_string(),
            arch: None,
            os: None,
            prebuilt: None,
        };

        assert!(!build.is_multi_os());
    }
}
