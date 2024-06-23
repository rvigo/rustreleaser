use serde::Serialize;

#[derive(Serialize)]
pub struct TreeItemRequest {
    pub path: String,
    pub mode: String,
    pub r#type: String,
    pub sha: String,
}

impl TreeItemRequest {
    pub fn new(
        path: impl Into<String>,
        mode: impl Into<String>,
        r#type: impl Into<String>,
        sha: impl Into<String>,
    ) -> Self {
        TreeItemRequest {
            path: path.into(),
            mode: mode.into(),
            r#type: r#type.into(),
            sha: sha.into(),
        }
    }
}

#[derive(Serialize)]
pub struct TreeRequest {
    pub base_tree: String,
    pub tree: Vec<TreeItemRequest>,
}

impl TreeRequest {
    pub fn new(base_tree: impl Into<String>, tree: Vec<TreeItemRequest>) -> Self {
        TreeRequest {
            base_tree: base_tree.into(),
            tree,
        }
    }
}
