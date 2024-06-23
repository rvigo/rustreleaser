mod blob;
mod commit;
mod object;
mod parent_sha;
mod pull_request_response;
mod release_response;
mod sha_response;
mod tree;
mod update_ref;
mod upsert_file_response;

pub use blob::BlobResponse;
pub use commit::CommitResponse;
pub use parent_sha::ParentShaResponse;
pub use pull_request_response::PullRequest;
pub use release_response::ReleaseResponse;
pub use sha_response::CommitShaResponse;
pub use sha_response::FileShaResponse;
pub use tree::TreeResponse;
pub use update_ref::UpdateRefResponse;

use crate::Result;
use reqwest::StatusCode;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize)]
pub struct Url(String);

#[derive(Serialize_repr, Deserialize_repr, Clone, Debug)]
#[repr(u16)]
#[non_exhaustive]
/// The HTTP response type.
pub enum ResponseType {
    /// Read the response as JSON
    Json = 1,
    /// Read the response as text
    Text,
    /// Read the response as binary
    Binary,
}

#[derive(Debug)]
pub struct Response(pub ResponseType, pub reqwest::Response);

impl Response {
    /// Get the [`StatusCode`] of this Response.
    pub fn status(&self) -> StatusCode {
        self.1.status()
    }

    /// Reads the response as raw bytes.
    pub async fn bytes(self) -> Result<RawResponse> {
        if let Err(err) = self.1.error_for_status_ref() {
            return self.extract_inner_error(err).await;
        }
        let status = self.status().as_u16();
        let data = self.1.bytes().await?.to_vec();
        Ok(RawResponse { status, data })
    }

    /// Reads the response.
    ///
    /// Note that the body is serialized to a [`Value`].
    pub async fn read(self) -> Result<ResponseData> {
        if let Err(err) = self.1.error_for_status_ref() {
            return self.extract_inner_error(err).await;
        }
        let url = Url(self.1.url().clone().as_str().to_owned());
        let mut headers = HashMap::new();
        let mut raw_headers = HashMap::new();
        for (name, value) in self.1.headers() {
            headers.insert(
                name.as_str().to_string(),
                String::from_utf8(value.as_bytes().to_vec())?,
            );
            raw_headers.insert(
                name.as_str().to_string(),
                self.1
                    .headers()
                    .get_all(name)
                    .into_iter()
                    .map(|v| String::from_utf8(v.as_bytes().to_vec()).map_err(Into::into))
                    .collect::<Result<Vec<String>>>()?,
            );
        }
        let status = self.1.status().as_u16();

        let data = match self.0 {
            ResponseType::Json => self.1.json().await?,
            ResponseType::Text => Value::String(self.1.text().await?),
            ResponseType::Binary => serde_json::to_value(self.1.bytes().await?.to_vec())?,
        };

        Ok(ResponseData {
            url,
            status,
            headers,
            raw_headers,
            data,
        })
    }

    pub async fn collect<T>(self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        if let Err(err) = self.1.error_for_status_ref() {
            return self.extract_inner_error(err).await;
        }

        let json = self.1.json::<T>().await?;

        Ok(json)
    }

    async fn extract_inner_error<T>(self, err: reqwest::Error) -> Result<T> {
        let original_error_message = self.1.text().await?;
        match ResponseError::try_from(original_error_message.to_owned()) {
            Ok(err) => Err(err.into()),
            Err(_) => Err(ResponseError::Detailed {
                status: err.status().map(|s| s.as_u16()).unwrap_or(500),
                message: original_error_message,
                details: Some(err.to_string().into()),
            }
            .into()),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ResponseData {
    /// Response URL. Useful if it followed redirects.
    pub url: Url,
    /// Response status code.
    pub status: u16,
    /// Response headers.
    pub headers: HashMap<String, String>,
    /// Response raw headers.
    pub raw_headers: HashMap<String, Vec<String>>,
    /// Response data.
    pub data: Value,
}

/// A response with raw bytes.
#[non_exhaustive]
#[derive(Debug)]
pub struct RawResponse {
    /// Response status code.
    pub status: u16,
    /// Response bytes.
    pub data: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum ResponseError {
    #[error("message: {message}, status_code: {status}, details: {details:?}")]
    Detailed {
        status: u16,
        message: String,
        details: Option<Value>,
    },
    #[error("{0} already exists")]
    AlreadyExists(What),
}

impl TryFrom<String> for ResponseError {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.contains("already exists") {
            let what = What::try_from(value).unwrap();
            Ok(ResponseError::AlreadyExists(what))
        } else {
            Err(())
        }
    }
}

#[derive(Debug)]
pub enum What {
    PullRequest,
    Reference,
}

impl std::fmt::Display for What {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            What::PullRequest => write!(f, "Pull Request"),
            What::Reference => write!(f, "Reference"),
        }
    }
}

impl TryFrom<String> for What {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.to_lowercase().contains("pull request") {
            Ok(What::PullRequest)
        } else if value.to_lowercase().contains("reference") {
            Ok(What::Reference)
        } else {
            Err(())
        }
    }
}
