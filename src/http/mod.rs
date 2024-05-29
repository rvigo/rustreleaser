pub mod error_response;

use self::error_response::ErrorResponse;
use anyhow::Result;
use reqwest::Client;
use std::ops::{Deref, DerefMut};

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

pub trait ResponseHandler {
    async fn handle(self) -> Result<String, ErrorResponse>;
}

impl ResponseHandler for Result<reqwest::Response, reqwest::Error> {
    async fn handle(self) -> Result<String, ErrorResponse> {
        match self {
            Ok(response) => {
                let status = response.status();
                let message = response
                    .text()
                    .await
                    .map_err(|e| ErrorResponse::new(e.to_string(), status.as_u16()))?;
                log::debug!("Response status: {}", status);

                if status.is_success() {
                    Ok(message)
                } else {
                    log::warn!("Response message: {}", message);
                    Ok(message)
                }
            }

            Err(error) => Err(ErrorResponse::internal_server_error(
                if error.to_string().is_empty() {
                    None
                } else {
                    Some(error.to_string())
                },
            )),
        }
    }
}
