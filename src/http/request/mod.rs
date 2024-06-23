mod assignees_request;
mod branch_ref_request;
mod commit;
mod committer_request;
mod create_release_request;
mod git_blob;
mod labels_request;
mod pull_request_request;
mod tree;
mod update_ref;
mod upsert_file_request;

pub use branch_ref_request::BranchRefRequest;
pub use commit::CommitRequest;
pub use create_release_request::CreateReleaseRequest;
pub use git_blob::Blob;
pub use git_blob::Encoding;
pub use pull_request_request::PullRequestRequest;
pub use tree::{TreeItemRequest, TreeRequest};
pub use update_ref::UpdateRefRequest;
pub use upsert_file_request::UpsertFileRequest;

use anyhow::Result;
use serde::Serialize;
use serde_json::Value;

pub trait SerializeRequest {
    fn into_value(self) -> Result<Value>
    where
        Self: Serialize + Sized,
    {
        let body = serde_json::to_value(&self)?;

        Ok(body)
    }
}

impl<T> SerializeRequest for T where T: Serialize {}

use super::{client::HeaderMap, response::ResponseType};
use reqwest::header::AUTHORIZATION;
use serde::Deserialize;
use std::{collections::HashMap, fmt};

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", content = "payload")]
#[non_exhaustive]
pub enum Body {
    /// A JSON body.
    Json(Value),
    /// A text string body.
    Text(String),
    /// A byte array body.
    Bytes(Vec<u8>),
}

impl Body {
    /// Creates a new JSON body.
    pub fn json<T: SerializeRequest + Serialize>(value: T) -> anyhow::Result<Self> {
        Ok(Body::Json(value.into_value()?))
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HttpRequestBuilder {
    /// The request method (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS, CONNECT or TRACE)
    pub method: Method,
    /// The request URL
    pub url: String,
    /// The request query params
    pub query: Option<HashMap<String, String>>,
    /// The request headers
    pub headers: Option<HeaderMap>,
    /// The request body
    pub body: Option<Body>,
    /// The response type (defaults to Json)
    pub response_type: Option<ResponseType>,
}

#[derive(Debug, Deserialize, Clone)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

impl From<Method> for reqwest::Method {
    fn from(method: Method) -> Self {
        match method {
            Method::Get => reqwest::Method::GET,
            Method::Post => reqwest::Method::POST,
            Method::Put => reqwest::Method::PUT,
            Method::Delete => reqwest::Method::DELETE,
            Method::Patch => reqwest::Method::PATCH,
        }
    }
}

impl HttpRequestBuilder {
    /// Initializes a new instance of the HttpRequestrequest_builder.
    pub fn new(method: Method, url: impl Into<String>) -> Self {
        Self {
            method,
            url: url.into(),
            query: None,
            headers: None,
            body: None,
            response_type: None,
        }
    }

    pub fn header(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.headers
            .get_or_insert_with(Default::default)
            .insert(key.into(), value.into());

        self
    }

    pub fn body(&mut self, body: Body) -> &mut Self {
        self.body = Some(body);
        self
    }

    pub fn bearer_auth<T>(&mut self, token: T) -> &mut Self
    where
        T: fmt::Display,
    {
        let header_value = format!("Bearer {}", token);

        self.headers
            .get_or_insert_with(Default::default)
            .insert(AUTHORIZATION.to_string(), header_value);

        self
    }

    pub fn build(self) -> HttpRequestBuilder {
        self
    }
}
