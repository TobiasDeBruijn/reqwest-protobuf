#![doc = include_str!("../README.md")]

use prost::{EncodeError, Message};
use async_trait::async_trait;
use reqwest::{RequestBuilder, Response};
use thiserror::Error;

/// Extension trait for the RequestBuilder
pub trait ProtobufRequestExt where Self: Sized {
    /// Configure the request to accept a protobuf response.
    /// Sets the `Accept` header to `application/protobuf`
    fn accept_protobuf(self) -> Self;

    /// Set the request payload encoded as protobuf.
    /// Sets the `Content-Type` header to `application/protobuf`
    fn protobuf<T: Message + Default>(self, value: T) -> Result<Self, EncodeError>;
}

/// Decode errors
#[derive(Debug, Error)]
pub enum DecodeError {
    /// Failed to extract the request body bytes
    #[error("Failed to extract body bytes: {0}")]
    Reqwest(#[from] reqwest::Error),
    /// Failed to decode the received bytes
    #[error("Failed to decode protobuf: {0:?}")]
    ProstDecode(prost::DecodeError)
}

impl From<prost::DecodeError> for DecodeError {
    fn from(x: prost::DecodeError) -> Self {
        Self::ProstDecode(x)
    }
}


/// Extension trait for the Response
#[async_trait]
pub trait ProtobufResponseExt {
    /// Get the response body decoded from Protobuf
    async fn protobuf<T: Message + Default>(self) -> Result<T, DecodeError>;
}

impl ProtobufRequestExt for RequestBuilder {
    fn accept_protobuf(self) -> Self {
        self.header("Accept", "application/protobuf")
    }

    fn protobuf<T: Message + Default>(self, value: T) -> Result<Self, EncodeError> {
        let mut buf = Vec::new();
        value.encode(&mut buf)?;
        let this = self.header("Content-Type", "application/protobuf");
        Ok(this.body(buf))
    }
}

#[async_trait]
impl ProtobufResponseExt for Response {
    async fn protobuf<T: Message + Default>(self) -> Result<T, DecodeError> {
        let body = self.bytes().await?;
        let decoded = T::decode(body)?;
        Ok(decoded)
    }
}
