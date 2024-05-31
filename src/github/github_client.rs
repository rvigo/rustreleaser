use super::{
    dto::{pull_request_dto::PullRequestDto, release_dto::ReleaseDto},
    request::{
        assignees_request::AssigneesRequest, branch_ref_request::BranchRefRequest,
        create_release_request::CreateReleaseRequest, labels_request::LabelsRequest,
        pull_request_request::PullRequestRequest,
    },
    response::{
        pull_request_response::PullRequest, release_response::ReleaseResponse, sha_response::Sha,
    },
};
use crate::{
    get,
    git::tag::Tag,
    github::{
        asset::{Asset, UploadedAsset},
        dto::commit_info_dto::CommitInfoDto,
        release::Release,
        request::upsert_file_request::UpsertFileRequest,
    },
    post, put, upload_file,
};
use anyhow::{Context, Result};
use base64::{prelude::BASE64_STANDARD, Engine};
use once_cell::sync::Lazy;
use std::env;
use tokio::{fs::File, io::AsyncReadExt};

pub static GITHUB_TOKEN: Lazy<String> =
    Lazy::new(|| env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN must be set"));

static CLIENT: Lazy<GithubClient> = Lazy::new(|| GithubClient);

pub fn instance() -> &'static GithubClient {
    &CLIENT
}

const GITHUB_DEFAULT_URL: &str = "https://github.com";
const GITHUB_API_REPO_URL: &str = "https://api.github.com/repos";
const GITHUB_API_UPLOAD_URL: &str = "https://uploads.github.com/repos";

pub struct GithubClient;

/// Github client api internal implementation
impl GithubClient {
    pub(super) async fn upload_asset(
        &self,
        asset: &Asset,
        owner: impl Into<String>,
        tag: &Tag,
        repo: impl Into<String>,
        release_id: u64,
    ) -> Result<UploadedAsset> {
        let mut file = File::open(&asset.path).await?;
        let mut content = vec![];

        file.read_to_end(&mut content).await?;

        let owner = owner.into();
        let repo = repo.into();

        let uri = format!(
            "{}/{}/{}/releases/{}/assets?name={}",
            GITHUB_API_UPLOAD_URL, &owner, &repo, release_id, asset.name
        );

        upload_file!(uri, content)?;

        let asset_url = format!(
            "{}/{}/{}/releases/download/v{}/{}",
            GITHUB_DEFAULT_URL,
            &owner,
            &repo,
            tag.strip_v_prefix(),
            asset.name
        );
        log::debug!("creating uploaded asset for {}", asset.name);
        let uploaded_asset = self.create_uploaded_asset(asset, asset_url);
        log::debug!("uploaded asset created: {:#?}", uploaded_asset);

        Ok(uploaded_asset)
    }

    pub(super) async fn get_commit_sha(
        &self,
        owner: impl Into<String>,
        repo: impl Into<String>,
        base: impl Into<String>,
    ) -> Result<Sha> {
        let owner = owner.into();
        let repo = repo.into();
        let base = base.into();

        let uri = format!(
            "{}/{}/{}/commits/{}",
            GITHUB_API_REPO_URL, &owner, &repo, &base
        );

        let response = get!(&uri)?;

        let sha = Sha { sha: response };

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

        let request = BranchRefRequest::new(branch.to_string(), sha.to_string());

        let body: String = serde_json::to_string(&request)?;

        post!(&uri, body)?;

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

        let file_sha = get!(uri).context("failed to get Formula sha value")?;

        let sha = serde_json::from_str::<Sha>(&file_sha).unwrap_or_default();

        let body = if sha.sha.is_empty() {
            log::debug!("creating new file");

            let request = UpsertFileRequest::new(
                &commit_info.message,
                content,
                Some(head),
                None,
                commit_info.committer.to_owned().into(),
            );

            serde_json::to_string(&request)?
        } else {
            log::debug!("updating file");

            let request = UpsertFileRequest::new(
                &commit_info.message,
                content,
                Some(head),
                Some(sha.sha),
                commit_info.committer.to_owned().into(),
            );

            serde_json::to_string(&request)?
        };

        let uri = format!(
            "{}/{}/{}/contents/{}",
            GITHUB_API_REPO_URL, owner, repo, path
        );

        put!(uri, body)?;

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
        );
        let body: String = serde_json::to_string(&request)?;

        let response = post!(&uri, body)?;

        let pr: PullRequest = serde_json::from_str(&response)?;

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
        );

        let body: String = serde_json::to_string(&request)?;

        let response = post!(&uri, body)?;

        let release = serde_json::from_str::<ReleaseResponse>(&response)?;

        Ok(Release::new(
            release.id,
            release_dto.owner,
            release_dto.repo,
        ))
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

        let response = get!(&uri)?;
        let release = serde_json::from_str::<ReleaseResponse>(&response)?;

        Ok(Release::new(release.id, owner, repo))
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

        let request = AssigneesRequest::new(assignees);

        let body: String = serde_json::to_string(&request)?;

        post!(&uri, body)?;

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

        let request = LabelsRequest::new(labels);

        let body: String = serde_json::to_string(&request)?;

        post!(&uri, body)?;

        Ok(())
    }

    fn create_uploaded_asset(&self, asset: &Asset, url: impl Into<String>) -> UploadedAsset {
        UploadedAsset::new(
            asset.name.to_owned(),
            url.into(),
            asset
                .checksum
                .as_ref()
                .unwrap_or(&String::default())
                .to_owned(),
        )
    }
}
