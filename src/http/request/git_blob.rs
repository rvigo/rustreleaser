use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Encoding {
    Base64,
}

#[derive(Serialize)]

pub struct Blob {
    pub content: String,
    pub encoding: Encoding,
}

impl Blob {
    pub fn new(content: impl Into<String>, encoding: Encoding) -> Self {
        Blob {
            content: content.into(),
            encoding,
        }
    }
}
