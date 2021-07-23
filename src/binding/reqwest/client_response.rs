use reqwest_lib as reqwest;

use crate::binding::http::SPEC_VERSION_HEADER;
use crate::event::SpecVersion;
use crate::message::{
    BinaryDeserializer, BinarySerializer, Encoding, Error, MessageAttributeValue,
    MessageDeserializer, Result, StructuredDeserializer, StructuredSerializer,
};
use crate::{header_value_to_str, message, Event};
use async_trait::async_trait;
use bytes::Bytes;
use reqwest::header::HeaderMap;
use reqwest::Response;
use std::convert::TryFrom;

/// Wrapper for [`Response`] that implements [`MessageDeserializer`] trait.
pub struct ResponseDeserializer {
    headers: HeaderMap,
    body: Bytes,
}

impl ResponseDeserializer {
    pub fn new(headers: HeaderMap, body: Bytes) -> ResponseDeserializer {
        ResponseDeserializer { headers, body }
    }
}

impl BinaryDeserializer for ResponseDeserializer {
    fn deserialize_binary<R: Sized, V: BinarySerializer<R>>(self, mut visitor: V) -> Result<R> {
        if self.encoding() != Encoding::BINARY {
            return Err(message::Error::WrongEncoding {});
        }

        let spec_version = SpecVersion::try_from(
            self.headers
                .get(SPEC_VERSION_HEADER)
                .map(|a| header_value_to_str!(a))
                .unwrap()?,
        )?;

        visitor = visitor.set_spec_version(spec_version.clone())?;

        let attributes = spec_version.attribute_names();

        for (hn, hv) in self.headers.iter().filter(|(hn, _)| {
            let key = hn.as_str();
            SPEC_VERSION_HEADER.ne(key) && key.starts_with("ce-")
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

        if let Some(hv) = self.headers.get("content-type") {
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

impl StructuredDeserializer for ResponseDeserializer {
    fn deserialize_structured<R: Sized, V: StructuredSerializer<R>>(self, visitor: V) -> Result<R> {
        if self.encoding() != Encoding::STRUCTURED {
            return Err(message::Error::WrongEncoding {});
        }
        visitor.set_structured_event(self.body.to_vec())
    }
}

impl MessageDeserializer for ResponseDeserializer {
    fn encoding(&self) -> Encoding {
        match (
            #[allow(clippy::borrow_interior_mutable_const)]
            self.headers
                .get(reqwest::header::CONTENT_TYPE)
                .map(|a| header_value_to_str!(a))
                .map(|r| r.ok())
                .flatten()
                .map(|e| e.starts_with("application/cloudevents+json")),
            self.headers.get(SPEC_VERSION_HEADER),
        ) {
            (Some(true), _) => Encoding::STRUCTURED,
            (_, Some(_)) => Encoding::BINARY,
            _ => Encoding::UNKNOWN,
        }
    }
}

/// Method to transform an incoming [`Response`] to [`Event`].
pub async fn response_to_event(res: Response) -> Result<Event> {
    let h = res.headers().to_owned();
    let b = res.bytes().await.map_err(|e| Error::Other {
        source: Box::new(e),
    })?;

    MessageDeserializer::into_event(ResponseDeserializer::new(h, b))
}

/// Extension Trait for [`Response`] which acts as a wrapper for the function [`response_to_event()`].
///
/// This trait is sealed and cannot be implemented for types outside of this crate.
#[async_trait(?Send)]
pub trait ResponseExt: private::Sealed {
    /// Convert this [`Response`] to [`Event`].
    async fn into_event(self) -> Result<Event>;
}

#[async_trait(?Send)]
impl ResponseExt for Response {
    async fn into_event(self) -> Result<Event> {
        response_to_event(self).await
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
    use mockito::mock;
    use reqwest_lib as reqwest;

    use crate::{EventBuilder, EventBuilderV10};
    use chrono::Utc;
    use serde_json::json;

    #[tokio::test]
    async fn test_response() {
        let time = Utc::now();
        let url = mockito::server_url();
        let _m = mock("GET", "/")
            .with_status(200)
            .with_header("ce-specversion", "1.0")
            .with_header("ce-id", "0001")
            .with_header("ce-type", "example.test")
            .with_header("ce-source", "http://localhost")
            .with_header("ce-someint", "10")
            .with_header("ce-time", &time.to_rfc3339())
            .create();

        let expected = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            //TODO this is required now because the message deserializer implictly set default values
            // As soon as this defaulting doesn't happen anymore, we can remove it (Issues #40/#41)
            .time(time)
            .source("http://localhost")
            .extension("someint", "10")
            .build()
            .unwrap();

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
        let time = Utc::now();
        let j = json!({"hello": "world"});

        let url = mockito::server_url();
        let _m = mock("GET", "/")
            .with_status(200)
            .with_header("ce-specversion", "1.0")
            .with_header("ce-id", "0001")
            .with_header("ce-type", "example.test")
            .with_header("ce-source", "http://localhost/")
            .with_header("content-type", "application/json")
            .with_header("ce-someint", "10")
            .with_header("ce-time", &time.to_rfc3339())
            .with_body(j.to_string())
            .create();

        let expected = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            //TODO this is required now because the message deserializer implictly set default values
            // As soon as this defaulting doesn't happen anymore, we can remove it (Issues #40/#41)
            .time(time)
            .source("http://localhost/")
            .data("application/json", j.to_string().into_bytes())
            .extension("someint", "10")
            .build()
            .unwrap();

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
        let time = Utc::now();

        let j = json!({"hello": "world"});
        let expected = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            //TODO this is required now because the message deserializer implictly set default values
            // As soon as this defaulting doesn't happen anymore, we can remove it (Issues #40/#41)
            .time(time)
            .source("http://localhost")
            .data("application/json", j.clone())
            .extension("someint", "10")
            .build()
            .unwrap();

        let url = mockito::server_url();
        let _m = mock("GET", "/")
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
}
