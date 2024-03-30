pub struct PullRequestDto {
    pub owner: String,
    pub repo: String,
    pub title: String,
    pub head: String,
    pub base: String,
    pub pr_body: String,
    pub assignees: Vec<String>,
    pub labels: Vec<String>,
}

impl PullRequestDto {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        owner: impl Into<String>,
        repo: impl Into<String>,
        title: impl Into<String>,
        head: impl Into<String>,
        base: impl Into<String>,
        pr_body: impl Into<String>,
        assignees: Vec<String>,
        labels: Vec<String>,
    ) -> Self {
        PullRequestDto {
            owner: owner.into(),
            repo: repo.into(),
            title: title.into(),
            head: head.into(),
            base: base.into(),
            pr_body: pr_body.into(),
            assignees,
            labels,
        }
    }
}
