use super::headers;
use crate::event::SpecVersion;
use crate::message::{
    BinaryDeserializer, BinarySerializer, Encoding, MessageAttributeValue, MessageDeserializer,
    Result, StructuredDeserializer, StructuredSerializer,
};
use crate::{message, Event};
use actix_web::web::{Bytes, BytesMut};
use actix_web::{web, HttpMessage, HttpRequest};
use async_trait::async_trait;
use futures::future::LocalBoxFuture;
use futures::{FutureExt, StreamExt};
use std::convert::TryFrom;

/// Wrapper for [`HttpRequest`] that implements [`MessageDeserializer`] trait.
pub struct HttpRequestDeserializer<'a> {
    req: &'a HttpRequest,
    body: Bytes,
}

impl HttpRequestDeserializer<'_> {
    pub fn new(req: &HttpRequest, body: Bytes) -> HttpRequestDeserializer {
        HttpRequestDeserializer { req, body }
    }
}

impl<'a> BinaryDeserializer for HttpRequestDeserializer<'a> {
    fn deserialize_binary<R: Sized, V: BinarySerializer<R>>(self, mut visitor: V) -> Result<R> {
        if self.encoding() != Encoding::BINARY {
            return Err(message::Error::WrongEncoding {});
        }

        let spec_version = SpecVersion::try_from(
            self.req
                .headers()
                .get(headers::SPEC_VERSION_HEADER)
                .unwrap()
                .to_str()
                .unwrap(),
        )?;

        visitor = visitor.set_spec_version(spec_version.clone())?;

        let attributes = spec_version.attribute_names();

        for (hn, hv) in self.req.headers().iter().filter(|(hn, _)| {
            let key = hn.as_str();
            headers::SPEC_VERSION_HEADER.ne(key) && key.starts_with("ce-")
        }) {
            let name = &hn.as_str()["ce-".len()..];

            if attributes.contains(&name) {
                visitor = visitor.set_attribute(
                    name,
                    MessageAttributeValue::String(String::from(header_value_to_str!(hv)?)),
                )?
            } else {
                visitor = visitor.set_extension(
                    name,
                    MessageAttributeValue::String(String::from(header_value_to_str!(hv)?)),
                )?
            }
        }

        if let Some(hv) = self.req.headers().get("content-type") {
            visitor = visitor.set_attribute(
                "datacontenttype",
                MessageAttributeValue::String(String::from(header_value_to_str!(hv)?)),
            )?
        }

        if !self.body.is_empty() {
            visitor.end_with_data(self.body.to_vec())
        } else {
            visitor.end()
        }
    }
}

impl<'a> StructuredDeserializer for HttpRequestDeserializer<'a> {
    fn deserialize_structured<R: Sized, V: StructuredSerializer<R>>(self, visitor: V) -> Result<R> {
        if self.encoding() != Encoding::STRUCTURED {
            return Err(message::Error::WrongEncoding {});
        }
        visitor.set_structured_event(self.body.to_vec())
    }
}

impl<'a> MessageDeserializer for HttpRequestDeserializer<'a> {
    fn encoding(&self) -> Encoding {
        if self.req.content_type() == "application/cloudevents+json" {
            Encoding::STRUCTURED
        } else if self
            .req
            .headers()
            .contains_key(super::headers::SPEC_VERSION_HEADER)
        {
            Encoding::BINARY
        } else {
            Encoding::UNKNOWN
        }
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
    MessageDeserializer::into_event(HttpRequestDeserializer::new(req, bytes.freeze()))
        .map_err(actix_web::error::ErrorBadRequest)
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

/// Extention Trait for [`HttpRequest`] which acts as a wrapper for the function [`request_to_event()`].
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

    use crate::{EventBuilder, EventBuilderV10};
    use chrono::Utc;
    use serde_json::json;

    #[actix_rt::test]
    async fn test_request() {
        let time = Utc::now();
        let expected = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .source("http://localhost/")
            //TODO this is required now because the message deserializer implictly set default values
            // As soon as this defaulting doesn't happen anymore, we can remove it (Issues #40/#41)
            .time(time)
            .extension("someint", "10")
            .build()
            .unwrap();

        let (req, payload) = test::TestRequest::post()
            .insert_header(("ce-specversion", "1.0"))
            .insert_header(("ce-id", "0001"))
            .insert_header(("ce-type", "example.test"))
            .insert_header(("ce-source", "http://localhost/"))
            .insert_header(("ce-someint", "10"))
            .insert_header(("ce-time", time.to_rfc3339()))
            .to_http_parts();

        let resp = req.to_event(web::Payload(payload)).await.unwrap();
        assert_eq!(expected, resp);
    }

    #[actix_rt::test]
    async fn test_request_with_full_data() {
        let time = Utc::now();
        let j = json!({"hello": "world"});

        let expected = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .source("http://localhost")
            //TODO this is required now because the message deserializer implictly set default values
            // As soon as this defaulting doesn't happen anymore, we can remove it (Issues #40/#41)
            .time(time)
            .data("application/json", j.to_string().into_bytes())
            .extension("someint", "10")
            .build()
            .unwrap();

        let (req, payload) = test::TestRequest::post()
            .insert_header(("ce-specversion", "1.0"))
            .insert_header(("ce-id", "0001"))
            .insert_header(("ce-type", "example.test"))
            .insert_header(("ce-source", "http://localhost"))
            .insert_header(("ce-someint", "10"))
            .insert_header(("ce-time", time.to_rfc3339()))
            .insert_header(("content-type", "application/json"))
            .set_json(&j)
            .to_http_parts();

        let resp = req.to_event(web::Payload(payload)).await.unwrap();
        assert_eq!(expected, resp);
    }
}
