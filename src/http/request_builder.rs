use super::{
    client::Client,
    request::{Body, HttpRequestBuilder, Method},
    response::{Response, ResponseType},
};
use crate::github::github_client::GITHUB_TOKEN;
use anyhow::Result;
use reqwest::header::{ACCEPT, USER_AGENT};

pub trait RequestType {}
pub struct Get;
pub struct Post;
pub struct Put;
pub struct Patch;

impl RequestType for Get {}
impl RequestType for Post {}
impl RequestType for Put {}
impl RequestType for Patch {}

pub struct InitBuilder {}

impl InitBuilder {
    pub fn get(client: Client, url: impl Into<String>) -> RequestBuilder<Get> {
        RequestBuilder {
            _marker: std::marker::PhantomData,
            client,
            builder: HttpRequestBuilder::new(Method::Get, url),
        }
    }

    pub fn post(client: Client, url: impl Into<String>) -> RequestBuilder<Post> {
        RequestBuilder {
            client,
            _marker: std::marker::PhantomData,
            builder: HttpRequestBuilder::new(Method::Post, url),
        }
    }

    pub fn put(client: Client, url: impl Into<String>) -> RequestBuilder<Put> {
        RequestBuilder {
            client,
            _marker: std::marker::PhantomData,
            builder: HttpRequestBuilder::new(Method::Put, url),
        }
    }

    pub fn patch(client: Client, url: impl Into<String>) -> RequestBuilder<Patch> {
        RequestBuilder {
            client,
            _marker: std::marker::PhantomData,
            builder: HttpRequestBuilder::new(Method::Patch, url),
        }
    }
}

pub struct RequestBuilder<T: RequestType> {
    _marker: std::marker::PhantomData<T>,
    client: Client,
    pub builder: HttpRequestBuilder,
}

impl<T> RequestBuilder<T>
where
    T: RequestType,
{
    pub async fn send(self) -> Result<Response> {
        self.client.send(self.builder.build()).await
    }

    pub fn response_type(mut self, response_type: ResponseType) -> Self {
        self.builder.response_type = Some(response_type);
        self
    }

    pub fn json_content_headers(mut self) -> Self {
        self.builder
            .bearer_auth(GITHUB_TOKEN.to_string())
            .header(ACCEPT.as_str(), "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header(USER_AGENT.as_str(), "rustreleaser")
            .header("Content-Type", "application/json");

        self
    }

    pub fn sha_header(mut self) -> Self {
        self.builder
            .bearer_auth(GITHUB_TOKEN.to_string())
            .header(ACCEPT.as_str(), "application/vnd.github.VERSION.sha")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header(USER_AGENT.as_str(), "rustreleaser");

        self
    }
}

impl RequestBuilder<Post> {
    pub fn body(mut self, body: Body) -> Self {
        self.builder.body(body);
        self
    }
}

impl RequestBuilder<Put> {
    pub fn body(mut self, body: Body) -> Self {
        self.builder.body(body);
        self
    }
}

impl RequestBuilder<Patch> {
    pub fn body(mut self, body: Body) -> Self {
        self.builder.body(body);
        self
    }
}
