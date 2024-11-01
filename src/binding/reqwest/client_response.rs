use reqwest_lib as reqwest;

use crate::binding;
use crate::message::{Error, Result};
use crate::Event;
use async_trait::async_trait;
use http;
use http::header;
use reqwest::Response;

/// Method to transform an incoming [`Response`] to [`Event`].
pub async fn response_to_event(res: Response) -> Result<Event> {
    let h = res.headers().to_owned();
    let b = res.bytes().await.map_err(|e| Error::Other {
        source: Box::new(e),
    })?;
    binding::http::to_event(&h, b.to_vec())
}

/// Method to transform an incoming [`Response`] to a batched [`Vec<Event>`]
pub async fn response_to_events(res: Response) -> Result<Vec<Event>> {
    if res
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .filter(|&v| v.starts_with(binding::CLOUDEVENTS_BATCH_JSON_HEADER))
        .is_none()
    {
        return Err(Error::WrongEncoding {});
    }

    let bytes = res.bytes().await.map_err(|e| Error::Other {
        source: Box::new(e),
    })?;

    Ok(serde_json::from_slice(&bytes)?)
}

/// Extension Trait for [`Response`] which acts as a wrapper for the function [`response_to_event()`].
///
/// This trait is sealed and cannot be implemented for types outside of this crate.
#[async_trait(?Send)]
pub trait ResponseExt: private::Sealed {
    /// Convert this [`Response`] to [`Event`].
    async fn into_event(self) -> Result<Event>;
    /// Convert this [`Response`] to a batched [`Vec<Event>`].
    async fn into_events(self) -> Result<Vec<Event>>;
}

#[async_trait(?Send)]
impl ResponseExt for Response {
    async fn into_event(self) -> Result<Event> {
        response_to_event(self).await
    }

    async fn into_events(self) -> Result<Vec<Event>> {
        response_to_events(self).await
    }
}

// Sealing the ResponseExt
mod private {
    use reqwest_lib as reqwest;

    pub trait Sealed {}
    impl Sealed for reqwest::Response {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest_lib as reqwest;
    use std::vec;

    use crate::test::fixtures;

    #[tokio::test]
    async fn test_response() {
        let url = mockito::server_url();
        let _m = mockito::mock("GET", "/")
            .with_status(200)
            .with_header("ce-specversion", "1.0")
            .with_header("ce-id", "0001")
            .with_header("ce-type", "test_event.test_application")
            .with_header("ce-source", "http://localhost/")
            .with_header("ce-someint", "10")
            .create();

        let expected = fixtures::v10::minimal_string_extension();

        let client = reqwest::Client::new();
        let res = client
            .get(&url)
            .send()
            .await
            .unwrap()
            .into_event()
            .await
            .unwrap();

        assert_eq!(expected, res);
    }

    #[tokio::test]
    async fn test_response_with_full_data() {
        let url = mockito::server_url();
        let _m = mockito::mock("GET", "/")
            .with_status(200)
            .with_header("ce-specversion", "1.0")
            .with_header("ce-id", "0001")
            .with_header("ce-type", "test_event.test_application")
            .with_header("ce-source", "http://localhost/")
            .with_header("ce-subject", "cloudevents-sdk")
            .with_header("content-type", "application/json")
            .with_header("ce-string_ex", "val")
            .with_header("ce-int_ex", "10")
            .with_header("ce-bool_ex", "true")
            .with_header("ce-time", &fixtures::time().to_rfc3339())
            .with_body(fixtures::json_data().to_string())
            .create();

        let expected = fixtures::v10::full_binary_json_data_string_extension();

        let client = reqwest::Client::new();
        let res = client
            .get(&url)
            .send()
            .await
            .unwrap()
            .into_event()
            .await
            .unwrap();

        assert_eq!(expected, res);
    }

    #[tokio::test]
    async fn test_structured_response_with_full_data() {
        let expected = fixtures::v10::full_json_data_string_extension();

        let url = mockito::server_url();
        let _m = mockito::mock("GET", "/")
            .with_status(200)
            .with_header(
                "content-type",
                "application/cloudevents+json; charset=utf-8",
            )
            .with_body(serde_json::to_string(&expected).unwrap())
            .create();

        let client = reqwest::Client::new();
        let res = client
            .get(&url)
            .send()
            .await
            .unwrap()
            .into_event()
            .await
            .unwrap();

        assert_eq!(expected, res);
    }

    #[tokio::test]
    async fn test_batched_response() {
        let expected = vec![fixtures::v10::full_json_data_string_extension()];

        let url = mockito::server_url();
        let _m = mockito::mock("GET", "/")
            .with_status(200)
            .with_header(
                "content-type",
                "application/cloudevents-batch+json; charset=utf-8",
            )
            .with_body(serde_json::to_string(&expected).unwrap())
            .create();

        let client = reqwest::Client::new();
        let res = client
            .get(&url)
            .send()
            .await
            .unwrap()
            .into_events()
            .await
            .unwrap();

        assert_eq!(expected, res);
    }
}
