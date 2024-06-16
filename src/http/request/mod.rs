mod assignees_request;
mod branch_ref_request;
mod committer_request;
mod create_release_request;
mod labels_request;
mod pull_request_request;
mod upsert_file_request;

pub use assignees_request::AssigneesRequest;
pub use branch_ref_request::BranchRefRequest;
pub use create_release_request::CreateReleaseRequest;
pub use labels_request::LabelsRequest;
pub use pull_request_request::PullRequestRequest;
use serde::Serialize;
pub use upsert_file_request::UpsertFileRequest;

use anyhow::Result;

pub trait SerializeRequest {
    fn into_request(self) -> Result<String>
    where
        Self: Serialize + Sized,
    {
        let body = serde_json::to_string(&self)?;

        Ok(body)
    }
}

impl SerializeRequest for PullRequestRequest {}
impl SerializeRequest for CreateReleaseRequest {}
impl SerializeRequest for AssigneesRequest {}
impl SerializeRequest for LabelsRequest {}
impl SerializeRequest for BranchRefRequest {}
impl SerializeRequest for UpsertFileRequest {}
