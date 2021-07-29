use crate::binding::http::{to_event, Headers};
use crate::Event;
use actix_web::web::BytesMut;
use actix_web::{web, HttpRequest};
use async_trait::async_trait;
use futures::future::LocalBoxFuture;
use futures::{FutureExt, StreamExt};
use http::header::{AsHeaderName, HeaderName, HeaderValue};

/// Implement Headers for the actix HeaderMap
impl<'a> Headers<'a> for actix_web::http::HeaderMap {
    type Iterator = Box<dyn Iterator<Item = (&'a HeaderName, &'a HeaderValue)> + 'a>;
    fn get<K: AsHeaderName>(&self, key: K) -> Option<&HeaderValue> {
        self.get(key.as_str())
    }
    fn iter(&'a self) -> Self::Iterator {
        Box::new(self.iter())
    }
}

/// Method to transform an incoming [`HttpRequest`] to [`Event`].
pub async fn request_to_event(
    req: &HttpRequest,
    mut payload: web::Payload,
) -> std::result::Result<Event, actix_web::error::Error> {
    let mut bytes = BytesMut::new();
    while let Some(item) = payload.next().await {
        bytes.extend_from_slice(&item?);
    }
    to_event(req.headers(), bytes.to_vec()).map_err(actix_web::error::ErrorBadRequest)
}

/// So that an actix-web handler may take an Event parameter
impl actix_web::FromRequest for Event {
    type Config = ();
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, std::result::Result<Self, Self::Error>>;

    fn from_request(r: &HttpRequest, p: &mut actix_web::dev::Payload) -> Self::Future {
        let payload = web::Payload(p.take());
        let request = r.to_owned();
        async move { request_to_event(&request, payload).await }.boxed_local()
    }
}

/// Extension Trait for [`HttpRequest`] which acts as a wrapper for the function [`request_to_event()`].
///
/// This trait is sealed and cannot be implemented for types outside of this crate.
#[async_trait(?Send)]
pub trait HttpRequestExt: private::Sealed {
    /// Convert this [`HttpRequest`] into an [`Event`].
    async fn to_event(
        &self,
        mut payload: web::Payload,
    ) -> std::result::Result<Event, actix_web::error::Error>;
}

#[async_trait(?Send)]
impl HttpRequestExt for HttpRequest {
    async fn to_event(
        &self,
        payload: web::Payload,
    ) -> std::result::Result<Event, actix_web::error::Error> {
        request_to_event(self, payload).await
    }
}

mod private {
    // Sealing the RequestExt
    pub trait Sealed {}
    impl Sealed for actix_web::HttpRequest {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    use crate::test::fixtures;
    use crate::{EventBuilder, EventBuilderV10};
    use serde_json::json;
    #[actix_rt::test]
    async fn test_request() {
        let mut expected = fixtures::v10::minimal();
        expected.set_extension("someint", "10");

        let (req, payload) = test::TestRequest::post()
            .header("ce-specversion", "1.0")
            .header("ce-id", "0001")
            .header("ce-type", "test_event.test_application")
            .header("ce-source", "http://localhost/")
            .header("ce-someint", "10")
            .to_http_parts();

        let resp = req.to_event(web::Payload(payload)).await.unwrap();
        assert_eq!(expected, resp);
    }

    #[actix_rt::test]
    async fn test_request_with_full_data() {
        let j = json!({"hello": "world"});

        let expected = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .source("http://localhost")
            .data("application/json", j.to_string().into_bytes())
            .extension("someint", "10")
            .build()
            .unwrap();

        let (req, payload) = test::TestRequest::post()
            .header("ce-specversion", "1.0")
            .header("ce-id", "0001")
            .header("ce-type", "example.test")
            .header("ce-source", "http://localhost")
            .header("ce-someint", "10")
            .header("content-type", "application/json")
            .set_json(&j)
            .to_http_parts();

        let resp = req.to_event(web::Payload(payload)).await.unwrap();
        assert_eq!(expected, resp);
    }

    #[actix_rt::test]
    async fn test_structured_request_with_full_data() {
        let j = json!({"hello": "world"});
        let payload = json!({
            "specversion": "1.0",
            "id": "0001",
            "type": "example.test",
            "source": "http://localhost",
            "someint": "10",
            "datacontenttype": "application/json",
            "data": j
        });
        let bytes = serde_json::to_string(&payload).expect("Failed to serialize test data to json");

        let expected = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .source("http://localhost")
            .data("application/json", j)
            .extension("someint", "10")
            .build()
            .unwrap();

        let (req, payload) = test::TestRequest::post()
            .header("content-type", "application/cloudevents+json")
            .set_payload(bytes)
            .to_http_parts();

        let resp = req.to_event(web::Payload(payload)).await.unwrap();
        assert_eq!(expected, resp);
    }
}
