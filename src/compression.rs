use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
pub enum Compression {
    #[default]
    TarGz,
    Zip,
}
