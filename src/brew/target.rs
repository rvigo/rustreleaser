use super::BrewArch;
use crate::build::os::Os;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Targets(pub Vec<Target>);

impl Targets {
    pub fn inner_type(&self) -> &Target {
        self.0.first().unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MultiTarget {
    pub os: Os,
    pub archs: Vec<BrewArch>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SingleTarget {
    pub url: String,
    pub hash: String,
}

impl SingleTarget {
    pub fn new(url: impl Into<String>, hash: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            hash: hash.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Target {
    Single(SingleTarget),
    Multi(MultiTarget),
}
