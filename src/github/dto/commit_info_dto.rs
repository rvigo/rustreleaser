use crate::build::committer::Committer;

pub struct CommitInfoDto {
    pub message: String,
    pub committer: Committer,
}

impl CommitInfoDto {
    pub fn new(message: impl Into<String>, committer: &Committer) -> Self {
        Self {
            message: message.into(),
            committer: committer.to_owned(),
        }
    }
}
