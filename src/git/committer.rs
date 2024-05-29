#[derive(Debug, Clone)]
pub struct Committer {
    pub author: String,
    pub email: String,
}

impl Default for Committer {
    fn default() -> Self {
        Committer {
            author: "Rafael Vigo".to_string(),
            email: "19755627+rvigo@users.noreply.github.com".to_string(),
        }
    }
}
