pub mod request;
pub mod response;

use reqwest::Client;
use reqwest::{
    header::{ACCEPT, USER_AGENT},
    RequestBuilder,
};
use std::ops::{Deref, DerefMut};
use thiserror::Error;

use crate::github::github_client::GITHUB_TOKEN;

pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new() -> Self {
        HttpClient {
            client: Client::new(),
        }
    }
}

impl Deref for HttpClient {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl DerefMut for HttpClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}

pub trait Headers {
    fn default_headers(self) -> RequestBuilder;
}

impl Headers for RequestBuilder {
    fn default_headers(self) -> RequestBuilder {
        self.bearer_auth(GITHUB_TOKEN.to_string())
            .header(ACCEPT, "application/vnd.github.VERSION.sha")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header(USER_AGENT, "rustreleaser")
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{message}")]
    GenericResponseError { message: String },
    #[error("Failed to read response text")]
    ReadResponseTextError {
        #[source]
        cause: reqwest::Error,
    },
    #[error("Failed to parse response")]
    ParseResponseError {
        #[source]
        cause: serde_json::Error,
    },
}
