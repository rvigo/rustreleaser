use crate::github::tag::Tag;

pub struct ReleaseDto {
    pub owner: String,
    pub repo: String,
    pub tag: Tag,
    pub target_branch: String,
    pub release_name: String,
    pub draft: bool,
    pub prerelease: bool,
    pub body: String,
}

impl ReleaseDto {
    pub fn new(
        owner: impl Into<String>,
        repo: impl Into<String>,
        tag: Tag,
        target_branch: impl Into<String>,
        release_name: impl Into<String>,
        draft: bool,
        prerelease: bool,
        body: impl Into<String>,
    ) -> Self {
        ReleaseDto {
            owner: owner.into(),
            repo: repo.into(),
            tag,
            target_branch: target_branch.into(),
            release_name: release_name.into(),
            draft,
            prerelease,
            body: body.into(),
        }
    }
}
