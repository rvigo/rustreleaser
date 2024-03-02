pub mod arch;
pub mod committer;
pub mod compression;
pub mod os;

use self::compression::Compression;
use arch::Arch;
use os::Os;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Build {
    pub arch: Option<Vec<Arch>>,
    pub os: Option<Vec<Os>>,
    pub binary: String,
    #[serde(default)]
    pub compression: Compression,
}

impl Build {
    pub fn is_multi_target(&self) -> bool {
        self.is_multi_arch() || self.is_multi_os()
    }

    pub fn is_multi_arch(&self) -> bool {
        if let Some(archs) = &self.arch {
            !archs.is_empty()
        } else {
            false
        }
    }

    pub fn is_multi_os(&self) -> bool {
        if let Some(oss) = &self.os {
            !oss.is_empty()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::arch::Arch;
    use super::compression::Compression;
    use super::os::Os;
    use super::*;

    #[test]
    fn should_validate_if_multi_target() {
        let build = Build {
            binary: "binary".to_string(),
            compression: Compression::TarGz,
            arch: Some(vec![Arch::Amd64]),
            os: Some(vec![Os::UnknownLinuxGnu]),
        };

        assert!(build.is_multi_target());
    }

    #[test]
    fn should_validate_id_single_target() {
        let build = Build {
            binary: "binary".to_string(),
            compression: Compression::TarGz,
            arch: None,
            os: None,
        };

        assert!(!build.is_multi_target());
    }

    #[test]
    fn should_validate_if_multi_arch() {
        let build = Build {
            binary: "binary".to_string(),
            compression: Compression::TarGz,
            arch: Some(vec![Arch::Amd64]),
            os: None,
        };

        assert!(build.is_multi_arch());
    }

    #[test]
    fn should_validate_if_single_arch() {
        let build = Build {
            binary: "binary".to_string(),
            compression: Compression::TarGz,
            arch: None,
            os: None,
        };

        assert!(!build.is_multi_arch());
    }

    #[test]
    fn should_validate_if_multi_os() {
        let build = Build {
            binary: "binary".to_string(),
            compression: Compression::TarGz,
            arch: None,
            os: Some(vec![Os::UnknownLinuxGnu]),
        };

        assert!(build.is_multi_os());
    }

    #[test]
    fn should_validate_if_single_os() {
        let build = Build {
            binary: "binary".to_string(),
            compression: Compression::TarGz,
            arch: None,
            os: None,
        };

        assert!(!build.is_multi_os());
    }
}
