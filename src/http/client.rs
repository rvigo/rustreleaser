use super::{
    request::{Body, HttpRequestBuilder},
    request_builder::{Get, InitBuilder, Patch, Post, Put, RequestBuilder},
    response::{Response, ResponseType},
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

#[derive(Clone, Debug)]
pub struct Client(reqwest::Client);

impl Client {
    pub fn new() -> Client {
        Client(reqwest::Client::new())
    }

    pub async fn send(&self, request: HttpRequestBuilder) -> Result<Response> {
        let mut builder = self.0.request(request.method.into(), &request.url);

        if let Some(headers) = request.headers {
            builder = builder.headers(headers.into());
        }

        if let Some(body) = request.body {
            builder = match body {
                Body::Json(json) => builder.json(&json),
                Body::Text(text) => builder.body(text),
                Body::Bytes(bytes) => builder.body(bytes),
            }
        }

        let response = builder.build()?;

        let response = self.0.execute(response).await?;
        Ok(Response(
            request.response_type.unwrap_or(ResponseType::Json),
            response,
        ))
    }
}

pub trait ClientRequestBuilder {
    fn get(self, url: impl Into<String>) -> RequestBuilder<Get>;
    fn post(self, url: impl Into<String>) -> RequestBuilder<Post>;
    fn put(self, url: impl Into<String>) -> RequestBuilder<Put>;
    fn patch(self, url: impl Into<String>) -> RequestBuilder<Patch>;
}

impl ClientRequestBuilder for Client {
    fn get(self, url: impl Into<String>) -> RequestBuilder<Get> {
        InitBuilder::get(self, url)
    }

    fn post(self, url: impl Into<String>) -> RequestBuilder<Post> {
        InitBuilder::post(self, url)
    }

    fn put(self, url: impl Into<String>) -> RequestBuilder<Put> {
        InitBuilder::put(self, url)
    }

    fn patch(self, url: impl Into<String>) -> RequestBuilder<Patch> {
        InitBuilder::patch(self, url)
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct HeaderMap(pub HashMap<String, String>);

impl Deref for HeaderMap {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for HeaderMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<HeaderMap> for reqwest::header::HeaderMap {
    fn from(map: HeaderMap) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        for (key, value) in map.0 {
            headers.insert(
                reqwest::header::HeaderName::from_bytes(key.as_bytes()).unwrap(),
                reqwest::header::HeaderValue::from_str(&value).unwrap(),
            );
        }
        headers
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
