use super::object::Object;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct UpdateRefResponse {
    pub object: Object,
}
