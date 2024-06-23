use super::object::Object;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ParentShaResponse {
    pub object: Object,
}
