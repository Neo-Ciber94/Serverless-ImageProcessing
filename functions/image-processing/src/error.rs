use std::{
    future::{ready, Future},
    pin::Pin,
};

use lambda_http::{Body, IntoResponse, Response};
use reqwest::{header, StatusCode};
use thiserror::Error;

pub type ResponseFuture = Pin<Box<dyn Future<Output = Response<Body>> + Send>>;

#[derive(Debug, Error)]
#[error("{message}")]
pub struct ResponseError {
    message: String,
    status: StatusCode,
}

impl ResponseError {
    pub fn new(status: StatusCode, msg: impl Into<String>) -> Self {
        ResponseError {
            message: msg.into(),
            status,
        }
    }

    pub fn from_error<E>(error: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, error.into().to_string())
    }

    pub fn with_status(self, status: StatusCode) -> Self {
        Self {
            status,
            message: self.message,
        }
    }

    pub fn status(&self) -> StatusCode {
        self.status
    }

    pub fn message(&self) -> &str {
        &self.message.as_str()
    }
}

impl IntoResponse for ResponseError {
    fn into_response(self) -> ResponseFuture {
        let ResponseError { message, status } = self;
        let msg = serde_json::json!({ "message": message });
        let json = serde_json::to_string(&msg).expect("failed to convert message to JSON");
        let body = Body::Text(json);

        Box::pin(ready(
            Response::builder()
                .header(
                    header::CONTENT_TYPE,
                    header::HeaderValue::from_static("application/json"),
                )
                .status(status)
                .body(body)
                .expect("unable to build http::Response"),
        ))
    }
}
