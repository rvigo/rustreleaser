use crate::config::DependsOnConfig;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Symbol {
    Build,
    Test,
}

#[derive(Serialize, Deserialize)]
pub struct DependsOn {
    pub name: String,
    pub symbol: Vec<Symbol>,
}

impl From<DependsOnConfig> for DependsOn {
    fn from(depends_on: DependsOnConfig) -> Self {
        DependsOn {
            name: depends_on.name,
            symbol: depends_on.symbol.into(),
        }
    }
}

impl From<String> for Symbol {
    fn from(s: String) -> Self {
        match s.as_str() {
            "build" => Symbol::Build,
            "test" => Symbol::Test,
            _ => panic!("Unknown symbol: {}", s),
        }
    }
}

impl ToString for Symbol {
    fn to_string(&self) -> String {
        match self {
            Symbol::Build => "build".to_string(),
            Symbol::Test => "test".to_string(),
        }
    }
}
