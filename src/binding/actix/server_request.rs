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
    use serde_json::json;
    #[actix_rt::test]
    async fn test_request() {
        let expected = fixtures::v10::minimal_string_extension();

        let (req, payload) = test::TestRequest::post()
            .insert_header(("ce-specversion", "1.0"))
            .insert_header(("ce-id", "0001"))
            .insert_header(("ce-type", "test_event.test_application"))
            .insert_header(("ce-source", "http://localhost/"))
            .insert_header(("ce-someint", "10"))
            .to_http_parts();

        let resp = req.to_event(web::Payload(payload)).await.unwrap();
        assert_eq!(expected, resp);
    }

    #[actix_rt::test]
    async fn test_request_with_full_data() {
        let expected = fixtures::v10::full_binary_json_data_string_extension();

        let (req, payload) = test::TestRequest::post()
            .insert_header(("ce-specversion", "1.0"))
            .insert_header(("ce-id", "0001"))
            .insert_header(("ce-type", "test_event.test_application"))
            .insert_header(("ce-subject", "cloudevents-sdk"))
            .insert_header(("ce-source", "http://localhost/"))
            .insert_header(("ce-time", fixtures::time().to_rfc3339()))
            .insert_header(("ce-string_ex", "val"))
            .insert_header(("ce-int_ex", "10"))
            .insert_header(("ce-bool_ex", "true"))
            .insert_header(("content-type", "application/json"))
            .set_json(&fixtures::json_data())
            .to_http_parts();

        let resp = req.to_event(web::Payload(payload)).await.unwrap();
        assert_eq!(expected, resp);
    }

    #[actix_rt::test]
    async fn test_structured_request_with_full_data() {
        let payload = json!({
            "specversion": "1.0",
            "id": "0001",
            "type": "test_event.test_application",
            "subject": "cloudevents-sdk",
            "source": "http://localhost/",
            "time": fixtures::time().to_rfc3339(),
            "string_ex": "val",
            "int_ex": "10",
            "bool_ex": "true",
            "datacontenttype": "application/json",
            "data": fixtures::json_data()
        });
        let bytes = serde_json::to_string(&payload).expect("Failed to serialize test data to json");

        let expected = fixtures::v10::full_json_data_string_extension();

        let (req, payload) = test::TestRequest::post()
            .insert_header(("content-type", "application/cloudevents+json"))
            .set_payload(bytes)
            .to_http_parts();

        let resp = req.to_event(web::Payload(payload)).await.unwrap();
        assert_eq!(expected, resp);
    }
}
