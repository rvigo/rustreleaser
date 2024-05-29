use serde::{Deserialize, Serialize};

use crate::git::committer::Committer;

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitterRequest {
    pub name: String,
    pub email: String,
}

impl From<Committer> for CommitterRequest {
    fn from(committer: Committer) -> Self {
        Self {
            name: committer.author,
            email: committer.email,
        }
    }
}
