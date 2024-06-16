use super::dto::{pull_request_dto::PullRequestDto, release_dto::ReleaseDto};
use crate::{
    git::tag::Tag,
    github::{dto::commit_info_dto::CommitInfoDto, release::Release},
    http::{
        request::{
            AssigneesRequest, BranchRefRequest, CreateReleaseRequest, LabelsRequest,
            PullRequestRequest, SerializeRequest, UpsertFileRequest,
        },
        response::{
            AsyncFrom, Bytes, CommitShaResponse, FileShaResponse, Json, PullRequest, Raw,
            ReleaseResponse, Response, UpsertFileResponse,
        },
        Error, Headers, HttpClient,
    },
};
use anyhow::{Context, Result};
use base64::{prelude::BASE64_STANDARD, Engine};
use once_cell::sync::Lazy;
use reqwest::header::CONTENT_TYPE;
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
        owner: impl Into<String>,
        repo: impl Into<String>,
        base: impl Into<String>,
    ) -> Result<CommitShaResponse> {
        let owner = owner.into();
        let repo = repo.into();
        let base = base.into();

        let uri = format!(
            "{}/{}/{}/commits/{}",
            GITHUB_API_REPO_URL, &owner, &repo, &base
        );

        let response = HttpClient::new().get(&uri).default_headers().send().await?;
        let response = Response::<Raw, String>::async_from(response).await;

        let sha = CommitShaResponse {
            sha: response.collect().map_err(Error::from)?,
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

        let request = BranchRefRequest::new(branch, sha).into_request()?;
        let response = HttpClient::new()
            .post(&uri)
            .default_headers()
            .body(request)
            .send()
            .await?;
        let response = Response::<Raw, String>::async_from(response).await;

        if let Some(error) = response.err() {
            log::debug!("Error creating branch: {}", error.message);
        }

        Ok(())
    }

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
        let response = HttpClient::new().get(uri).default_headers().send().await?;
        let response = Response::<Json, FileShaResponse>::async_from(response).await;

        let sha = response.collect().unwrap_or_default();

        let body = if sha.sha.is_empty() {
            log::debug!("sha is empyt, creating new file");

            UpsertFileRequest::new(
                &commit_info.message,
                content,
                Some(head),
                None,
                commit_info.committer.to_owned().into(),
            )
        } else {
            log::debug!("found sha, updating file");
            log::debug!("remote sha: {}", sha.sha);
            UpsertFileRequest::new(
                &commit_info.message,
                content,
                Some(head),
                Some(sha.sha),
                commit_info.committer.to_owned().into(),
            )
        };

        let body = body.into_request()?;
        let uri = format!(
            "{}/{}/{}/contents/{}",
            GITHUB_API_REPO_URL, owner, repo, path
        );

        let response = HttpClient::new()
            .put(&uri)
            .default_headers()
            .header(CONTENT_TYPE, "application/octet-stream")
            .body(body)
            .send()
            .await
            .context("error commiting the file")?;

        let response = Response::<Json, UpsertFileResponse>::async_from(response).await;
        response.collect()?;

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
            pull_request.title,
            pull_request.head,
            pull_request.base,
            pull_request.pr_body,
        )
        .into_request()?;
        let response = HttpClient::new()
            .post(uri)
            .default_headers()
            .body(request)
            .send()
            .await?;

        let response = Response::<Json, PullRequest>::async_from(response).await;

        let pr: PullRequest = response
            .collect()
            .context("Cannot collect json from `Create Pull Request` response")?;

        if !pull_request.assignees.is_empty() {
            self.set_pr_assignees(
                &pull_request.owner,
                &pull_request.repo,
                pr.number,
                pull_request.assignees,
            )
            .await?;
        }

        if !pull_request.labels.is_empty() {
            self.set_pr_labels(
                &pull_request.owner,
                &pull_request.repo,
                pr.number.to_string(),
                pull_request.labels,
            )
            .await?;
        }

        Ok(pr)
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
        .into_request()?;
        let response = HttpClient::new()
            .post(uri)
            .default_headers()
            .body(request)
            .send()
            .await?;
        let response = Response::<Json, ReleaseResponse>::async_from(response).await;

        let release: ReleaseResponse = response
            .collect()
            .context("Cannot collect json from `Create Release` response")?;

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
        let response = HttpClient::new().get(url).default_headers().send().await?;

        let response = Response::<Bytes, Vec<u8>>::async_from(response).await;
        let bytes = response.collect_bytes()?;
        Ok(bytes.to_vec())
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

        let response = HttpClient::new().get(uri).default_headers().send().await?;
        let response = Response::<Json, ReleaseResponse>::async_from(response).await;

        let release: ReleaseResponse = response
            .collect()
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

    async fn set_pr_assignees(
        &self,
        owner: impl Into<String>,
        repo: impl Into<String>,
        pr_number: u64,
        assignees: Vec<String>,
    ) -> Result<()> {
        let uri = format!(
            "{}/{}/{}/issues/{}/assignees",
            GITHUB_API_REPO_URL,
            owner.into(),
            repo.into(),
            pr_number
        );

        let request = AssigneesRequest::new(assignees).into_request()?;
        let response = HttpClient::new()
            .post(uri)
            .default_headers()
            .body(request)
            .send()
            .await?;
        let response = Response::<Json, String>::async_from(response).await;

        match response {
            Response::Success(response) => log::debug!("Assignees set: {}", response.payload),
            Response::Error(error) => {
                log::error!("Error setting assignees: {}", error.message)
            }
        };

        Ok(())
    }

    async fn set_pr_labels(
        &self,
        owner: impl Into<String>,
        repo: impl Into<String>,
        pr_number: String,
        labels: Vec<String>,
    ) -> Result<()> {
        let uri = format!(
            "{}/{}/{}/issues/{}/labels",
            GITHUB_API_REPO_URL,
            owner.into(),
            repo.into(),
            pr_number
        );

        let request = LabelsRequest::new(labels).into_request()?;

        let response = HttpClient::new()
            .post(uri)
            .default_headers()
            .body(request)
            .send()
            .await?;
        let response = Response::<Json, String>::async_from(response).await;

        match response {
            Response::Success(response) => log::debug!("Labels set: {}", response.payload),
            Response::Error(error) => {
                log::error!("Error setting labels: {}", error.message)
            }
        };

        Ok(())
    }
}
