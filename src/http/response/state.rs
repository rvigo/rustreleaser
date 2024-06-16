use crate::http::Error;

use anyhow::Result;
use serde::de::DeserializeOwned;
use std::{marker::PhantomData, ops::Deref};

pub struct ErrorResponse {
    pub message: String,
}

pub struct Raw;

pub struct Json;

pub struct Bytes(pub Vec<u8>);

impl Deref for Bytes {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait ResponseType {}

impl ResponseType for Raw {}
impl ResponseType for Json {}
impl ResponseType for Bytes {}

pub enum Response<ResponseType, T> {
    Success(Inner<ResponseType, T>),
    Error(ErrorResponse),
}

pub struct Inner<ResponseType, T> {
    marker: PhantomData<ResponseType>,
    pub payload_str: String,
    pub payload: T,
    pub status: u16,
}

impl<S, T> Response<S, T>
where
    S: ResponseType,
{
    pub fn err(&self) -> Option<&ErrorResponse> {
        match self {
            Response::Success(_) => None,
            Response::Error(inner) => Some(inner),
        }
    }

    pub fn collect(self) -> Result<T, Error> {
        match self {
            Response::Success(response) => Ok(response.payload),
            Response::Error(response) => Err(Error::GenericResponseError {
                message: response.message,
            }),
        }
    }
}

impl Response<Bytes, Vec<u8>> {
    pub fn collect_bytes(self) -> Result<Bytes, Error> {
        match self {
            Response::Success(response) => Ok(Bytes(response.payload)),
            Response::Error(response) => Err(Error::GenericResponseError {
                message: response.message,
            }),
        }
    }
}

pub trait AsyncFrom<T>: Sized {
    async fn async_from(value: T) -> Self;
}

impl<T> AsyncFrom<reqwest::Response> for Response<Json, T>
where
    T: DeserializeOwned,
{
    async fn async_from(value: reqwest::Response) -> Self {
        let status = value.status().as_u16();

        let (payload, text) = match value.text().await {
            Ok(text) => {
                if !(200..300).contains(&status) {
                    return Response::Error(ErrorResponse { message: text });
                }
                let payload = match serde_json::from_str::<T>(&text) {
                    Ok(payload) => payload,
                    Err(err) => {
                        return Response::Error(ErrorResponse {
                            message: format!("Failed to parse json: {}", err),
                        })
                    }
                };
                (payload, text)
            }
            Err(err) => {
                return Response::Error(ErrorResponse {
                    message: format!("Failed to read response text: {}", err),
                });
            }
        };
        Response::Success(Inner {
            marker: PhantomData,
            status,
            payload,
            payload_str: text,
        })
    }
}

impl AsyncFrom<reqwest::Response> for Response<Raw, String> {
    async fn async_from(value: reqwest::Response) -> Self {
        let status = value.status().as_u16();

        let payload = match value.text().await {
            Ok(text) => {
                if !(200..300).contains(&status) {
                    return Response::Error(ErrorResponse { message: text });
                }

                text
            }
            Err(err) => {
                return Response::Error(ErrorResponse {
                    message: format!("Failed to read response text: {}", err),
                });
            }
        };
        Response::Success(Inner {
            marker: PhantomData,
            status,
            payload: payload.to_owned(),
            payload_str: payload,
        })
    }
}

impl AsyncFrom<reqwest::Response> for Response<Bytes, Vec<u8>> {
    async fn async_from(value: reqwest::Response) -> Self {
        let status = value.status().as_u16();

        let payload = match value.bytes().await {
            Ok(bytes) => {
                if !(200..300).contains(&status) {
                    return Response::Error(ErrorResponse {
                        message: "Failed to read response bytes".to_string(),
                    });
                }

                bytes.to_vec()
            }
            Err(err) => {
                return Response::Error(ErrorResponse {
                    message: format!("Failed to read response bytes: {}", err),
                });
            }
        };
        Response::Success(Inner {
            marker: PhantomData,
            status,
            payload,
            payload_str: "".to_string(),
        })
    }
}
