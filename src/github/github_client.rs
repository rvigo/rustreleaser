use super::dto::{pull_request_dto::PullRequestDto, release_dto::ReleaseDto};
use crate::{
    git::tag::Tag,
    github::{dto::commit_info_dto::CommitInfoDto, release::Release},
    http::{
        client::{Client, ClientRequestBuilder},
        request::{
            Blob, Body, BranchRefRequest, CommitRequest, CreateReleaseRequest, Encoding,
            PullRequestRequest, SerializeRequest, TreeItemRequest, TreeRequest, UpdateRefRequest,
            UpsertFileRequest,
        },
        response::{
            BlobResponse, CommitResponse, CommitShaResponse, FileShaResponse, ParentShaResponse,
            PullRequest, ReleaseResponse, ResponseType, TreeResponse, UpdateRefResponse,
        },
    },
};
use anyhow::{Context, Result};
use base64::{prelude::BASE64_STANDARD, Engine};
use once_cell::sync::Lazy;
use std::env;

pub static GITHUB_TOKEN: Lazy<String> =
    Lazy::new(|| env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN must be set"));

static CLIENT: Lazy<GithubClient> = Lazy::new(|| GithubClient);

pub fn instance() -> &'static GithubClient {
    &CLIENT
}

const GITHUB_API_REPO_URL: &str = "https://api.github.com/repos";

pub struct GithubClient;

/// Github client api internal implementation
impl GithubClient {
    pub(super) async fn get_commit_sha(
        &self,
        owner: &str,
        repo: &str,
        base: &str,
    ) -> Result<CommitShaResponse> {
        let uri = format!(
            "{}/{}/{}/commits/{}",
            GITHUB_API_REPO_URL, &owner, &repo, &base
        );
        let response = Client::new()
            .get(uri)
            .sha_header()
            .response_type(ResponseType::Text)
            .send()
            .await?
            .read()
            .await?;

        let sha = response.data.as_str().expect("Cannot get sha");
        log::debug!("Commit sha: {}", sha);
        let sha = CommitShaResponse {
            sha: sha.to_owned(),
        };
        Ok(sha)
    }

    pub(super) async fn create_branch(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
        sha: &str,
    ) -> Result<()> {
        let uri = format!("{}/{}/{}/git/refs", GITHUB_API_REPO_URL, owner, repo);

        let request = BranchRefRequest::new(branch, sha).into_value()?;
        log::debug!("create ref request: {}", request);
        let response = Client::new()
            .post(uri)
            .json_content_headers()
            .body(Body::Json(request))
            .response_type(ResponseType::Text)
            .send()
            .await?
            .read()
            .await
            .context("Error creating branch")?;

        log::info!("Branch created: {}", response.data);

        Ok(())
    }

    pub(super) async fn create_pull_request(
        &self,
        pull_request: PullRequestDto,
    ) -> Result<PullRequest> {
        log::debug!("Creating pull request");
        let uri = format!(
            "{}/{}/{}/pulls",
            GITHUB_API_REPO_URL, pull_request.owner, pull_request.repo
        );

        let request = PullRequestRequest::new(
            &pull_request.title,
            &pull_request.head,
            &pull_request.base,
            &pull_request.pr_body,
        )
        .into_value()?;

        let response = Client::new()
            .post(uri)
            .json_content_headers()
            .response_type(ResponseType::Json)
            .body(Body::Json(request))
            .send()
            .await
            .context("Cannot collect json from `Create Pull Request` response")?;

        let response = response.collect::<PullRequest>().await?;

        log::debug!("Pull request created: {:?}", response);

        // let pr = HttpClient::new()
        //     .post(uri)
        //     .json_content_headers()
        //     .body(request_builder)
        //     .send()
        //     .as_json::<PullRequest>()
        //     .await
        //     .collect()
        //     .context("Cannot collect json from `Create Pull Request` response")?;

        // if !pull_request.assignees.is_empty() {
        //     self.set_pr_assignees(
        //         &pull_request.owner,
        //         &pull_request.repo,
        //         pr.number,
        //         pull_request.assignees,
        //     )
        //     .await?;
        // }

        // if !pull_request.labels.is_empty() {
        //     self.set_pr_labels(
        //         &pull_request.owner,
        //         &pull_request.repo,
        //         pr.number.to_string(),
        //         pull_request.labels,
        //     )
        //     .await?;
        // }

        Ok(response)
    }

    pub(super) async fn create_release(&self, release_dto: ReleaseDto) -> Result<Release> {
        let uri = format!(
            "{}/{}/{}/releases",
            GITHUB_API_REPO_URL, release_dto.owner, release_dto.repo
        );

        let request = CreateReleaseRequest::new(
            release_dto.tag.name(),
            release_dto.target_branch,
            release_dto.release_name,
            release_dto.body,
            release_dto.draft,
            release_dto.prerelease,
        )
        .into_value()?;
        let response = Client::new()
            .post(uri)
            .json_content_headers()
            .response_type(ResponseType::Json)
            .body(Body::Json(request))
            .send()
            .await?;
        let release = response.collect::<ReleaseResponse>().await?;

        Ok(Release::new(
            release.id,
            release_dto.owner,
            release_dto.repo,
            release.name,
            release.tarball_url,
            release.zipball_url,
            release.tag_name,
        ))
    }

    pub(super) async fn download_tarball(&self, url: &str) -> Result<Vec<u8>> {
        let response = Client::new()
            .get(url)
            .json_content_headers()
            .response_type(ResponseType::Binary)
            .send()
            .await?;

        let response = response
            .bytes()
            .await
            .context("Cannot get bytes from response")?;
        Ok(response.data)
    }

    pub(super) async fn get_release_by_tag(
        &self,
        owner: &str,
        repo: &str,
        tag: &Tag,
    ) -> Result<Release> {
        let uri = format!(
            "{}/{}/{}/releases/tags/{}",
            GITHUB_API_REPO_URL,
            owner,
            repo,
            tag.name()
        );

        let response = Client::new()
            .get(uri)
            .json_content_headers()
            .response_type(ResponseType::Json)
            .send()
            .await?;
        let release = response
            .collect::<ReleaseResponse>()
            .await
            .context("Cannot collect json from `Get Release` response")?;

        Ok(Release::new(
            release.id,
            owner,
            repo,
            release.name,
            release.tarball_url,
            release.zipball_url,
            release.tag_name,
        ))
    }

    // async fn set_pr_assignees(
    //     &self,
    //     owner: impl Into<String>,
    //     repo: impl Into<String>,
    //     pr_number: u64,
    //     assignees: Vec<String>,
    // ) -> Result<()> {
    //     let uri = format!(
    //         "{}/{}/{}/issues/{}/assignees",
    //         GITHUB_API_REPO_URL,
    //         owner.into(),
    //         repo.into(),
    //         pr_number
    //     );

    //     let request = AssigneesRequest::new(assignees).into_request()?;
    //     let response = HttpClient::new()
    //         .post(uri)
    //         .sha_header()
    //         .body(request)
    //         .send()
    //         .raw()
    //         .await;

    //     match response {
    //         Response::Success(response) => log::debug!("Assignees set: {}", response.payload),
    //         Response::Error(error) => {
    //             log::error!("Error setting assignees: {}", error.message())
    //         }
    //     };

    //     Ok(())
    // }

    // async fn set_pr_labels(
    //     &self,
    //     owner: impl Into<String>,
    //     repo: impl Into<String>,
    //     pr_number: String,
    //     labels: Vec<String>,
    // ) -> Result<()> {
    //     let uri = format!(
    //         "{}/{}/{}/issues/{}/labels",
    //         GITHUB_API_REPO_URL,
    //         owner.into(),
    //         repo.into(),
    //         pr_number
    //     );

    //     let request = LabelsRequest::new(labels).into_request()?;

    //     let response = HttpClient::new()
    //         .post(uri)
    //         .sha_header()
    //         .body(request)
    //         .send()
    //         .raw()
    //         .await;

    //     match response {
    //         Response::Success(response) => log::debug!("Labels set: {}", response.payload),
    //         Response::Error(error) => {
    //             log::error!("Error setting labels: {}", error.message())
    //         }
    //     };

    //     Ok(())
    // }
}

impl GithubClient {
    pub(super) async fn upsert_file(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
        content: &str,
        head: String,
        commit_info: CommitInfoDto,
    ) -> Result<()> {
        log::debug!("Upserting file");
        let content = BASE64_STANDARD.encode(content.as_bytes());

        let uri = &format!(
            "{}/{}/{}/contents/{}",
            GITHUB_API_REPO_URL, owner, repo, path
        );
        log::debug!("Getting file sha");
        let response = Client::new()
            .get(uri)
            .sha_header()
            .response_type(ResponseType::Json)
            .send()
            .await?;
        let sha = response.collect::<FileShaResponse>().await?;

        if sha.sha.is_empty() {
            self.insert_file(owner, repo, path, &content, head, commit_info)
                .await?;
        } else {
            self.update_file(owner, repo, path, &content, head, commit_info)
                .await?;
        }
        Ok(())
    }

    pub async fn insert_file(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
        content: &str,
        head: String,
        commit_info: CommitInfoDto,
    ) -> Result<()> {
        log::debug!("sha is empyt, creating new file");

        let body = UpsertFileRequest::new(
            &commit_info.message,
            content,
            Some(head),
            None,
            commit_info.committer.to_owned().into(),
        );

        let body = body.into_value()?;

        let uri = format!(
            "{}/{}/{}/contents/{}",
            GITHUB_API_REPO_URL, owner, repo, path
        );
        Client::new()
            .put(uri)
            .json_content_headers()
            .body(Body::Json(body))
            .send()
            .await
            .context("Error committing the file")?;

        Ok(())
    }

    pub(super) async fn update_file(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
        content: &str,
        head: String,
        commit_info: CommitInfoDto,
    ) -> Result<()> {
        log::debug!("Updating file");

        let blob = Blob::new(content, Encoding::Base64);
        let blob_uri = format!("{}/{}/{}/git/blobs", GITHUB_API_REPO_URL, owner, repo);
        let blob_response = Client::new()
            .post(blob_uri)
            .json_content_headers()
            .response_type(ResponseType::Json)
            .body(Body::json(blob)?)
            .send()
            .await
            .context("Error while request blob creation")?;

        let blob_response = blob_response.collect::<BlobResponse>().await?;
        log::debug!("blob response: {:?}", blob_response);
        let base_tree_uri = format!(
            "{}/{}/{}/git/trees/{}",
            GITHUB_API_REPO_URL, owner, repo, head
        );

        log::debug!("Getting base tree");

        let response = Client::new()
            .get(base_tree_uri)
            .sha_header()
            .response_type(ResponseType::Json)
            .send()
            .await?;
        let base_tree_response = response.collect::<TreeResponse>().await?;

        let base_tree_sha = base_tree_response.sha;

        let tree_item = TreeItemRequest::new(path, "100644", "blob", blob_response.sha);
        let tree = TreeRequest::new(base_tree_sha, vec![tree_item]);

        let tree_uri = format!("{}/{}/{}/git/trees", GITHUB_API_REPO_URL, owner, repo);

        log::debug!("Creating tree");

        let tree_response = Client::new()
            .post(tree_uri)
            .json_content_headers()
            .body(Body::json(tree)?)
            .response_type(ResponseType::Json)
            .send()
            .await?
            .collect::<TreeResponse>()
            .await
            .context("Cannot create a new tree")?;

        let tree_sha = tree_response.sha;

        let parent_sha_uri = format!(
            "{}/{}/{}/git/refs/heads/{}",
            GITHUB_API_REPO_URL, owner, repo, head
        );

        let parent_sha_response = Client::new()
            .get(parent_sha_uri)
            .sha_header()
            .response_type(ResponseType::Json)
            .send()
            .await?;
        let parent_sha_response = parent_sha_response.collect::<ParentShaResponse>().await?;

        let parent_sha = parent_sha_response.object.sha;

        let commit_uri = format!("{}/{}/{}/git/commits", GITHUB_API_REPO_URL, owner, repo);
        let commit_request =
            CommitRequest::new(&commit_info.message, tree_sha, vec![parent_sha.to_string()])
                .into_value()?;

        log::debug!("Creating commit");

        let commit_response = Client::new()
            .post(commit_uri)
            .json_content_headers()
            .body(Body::Json(commit_request))
            .send()
            .await?;
        let commit_response = commit_response.collect::<CommitResponse>().await?;

        log::debug!("updating head: {head}");
        let update_ref_uri = format!(
            "{}/{}/{}/git/refs/heads/{}",
            GITHUB_API_REPO_URL, owner, repo, head
        );

        let update_ref_request = UpdateRefRequest {
            sha: commit_response.sha.to_owned(),
        };
        let update_ref_response = Client::new()
            .patch(update_ref_uri)
            .json_content_headers()
            .body(Body::Json(update_ref_request.into_value()?))
            .response_type(ResponseType::Json)
            .send()
            .await?
            .collect::<UpdateRefResponse>()
            .await
            .context("error updating head")?;

        log::debug!(
            "Head updated? {}",
            update_ref_response.object.sha == commit_response.sha
        );
        Ok(())
    }
}
