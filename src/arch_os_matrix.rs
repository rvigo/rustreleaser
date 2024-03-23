use crate::build::{arch::Arch, os::Os, Build};

#[derive(Debug, Clone)]
pub struct ArchOsMatrixEntry {
    pub arch: Arch,
    pub os: Os,
}

impl ArchOsMatrixEntry {
    pub fn new(arch: &Arch, os: &Os) -> Self {
        Self {
            arch: arch.to_owned(),
            os: os.to_owned(),
        }
    }
}

pub trait PushArchOsMatrix {
    fn push_entry(&mut self, entry: ArchOsMatrixEntry);
}

impl PushArchOsMatrix for Vec<ArchOsMatrixEntry> {
    fn push_entry(&mut self, entry: ArchOsMatrixEntry) {
        self.push(entry);
    }
}

impl ToString for ArchOsMatrixEntry {
    fn to_string(&self) -> String {
        format!("{}-{}", self.arch.to_string(), self.os.to_string())
    }
}

impl From<Build> for Vec<ArchOsMatrixEntry> {
    fn from(build: Build) -> Self {
        let mut matrix = Vec::new();

        if let Some(archs) = build.arch {
            if let Some(oss) = &build.os {
                for arch in archs {
                    for os in oss {
                        matrix.push_entry(ArchOsMatrixEntry::new(&arch, &os));
                    }
                }
            }
        }

        matrix
    }
}
