use super::headers;
use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use cloudevents::event::SpecVersion;
use cloudevents::message::{
    BinaryDeserializer, BinarySerializer, Encoding, MessageAttributeValue, MessageDeserializer,
    Result, StructuredDeserializer, StructuredSerializer,
};
use cloudevents::{message, Event};
use std::collections::HashMap;
use std::convert::TryFrom;
use surf::{Error, Response};

/// Wrapper for [`Response`] that implements [`MessageDeserializer`] trait.
pub struct ResponseDeserializer {
    headers: HashMap<String, String>,
    body: Bytes,
}

impl ResponseDeserializer {
    pub fn new(headers: HashMap<String, String>, body: Bytes) -> ResponseDeserializer {
        ResponseDeserializer { headers, body }
    }
}

impl<'a> BinaryDeserializer for ResponseDeserializer {
    fn deserialize_binary<R: Sized, V: BinarySerializer<R>>(self, mut visitor: V) -> Result<R> {
        if self.encoding() != Encoding::BINARY {
            return Err(message::Error::WrongEncoding {});
        }

        let versionheader = match self.headers.get("ce-specversion") {
            Some(s) => s.as_str(),
            None => "",
        };
        let spec_version = SpecVersion::try_from(versionheader)?;

        visitor = visitor.set_spec_version(spec_version.clone())?;

        let attributes = spec_version.attribute_names();

        for (k, _) in self.headers.iter().filter(|&(k, _)| {
            headers::SPEC_VERSION_HEADER.ne(k.as_str()) && k.as_str().starts_with("ce-")
        }) {
            let name = &k.as_str()["ce-".len()..];

            if attributes.contains(&name) {
                visitor = visitor.set_attribute(
                    name,
                    MessageAttributeValue::String(String::from(header_to_str!(self
                        .headers
                        .get(k)))),
                )?
            } else {
                visitor = visitor.set_extension(
                    name,
                    MessageAttributeValue::String(String::from(header_to_str!(self
                        .headers
                        .get(k)))),
                )?
            }
        }

        if !self.body.is_empty() {
            // surf defaults the content-type header but we are ignoring it if the body is empty
            if let Some(hv) = self.headers.get("content-type") {
                println!("content-type: {}", hv);
                visitor = visitor.set_attribute(
                    "datacontenttype",
                    MessageAttributeValue::String(String::from(hv.as_str())),
                )?
            }
            visitor.end_with_data(self.body.to_vec())
        } else {
            visitor.end()
        }
    }
}

impl<'a> StructuredDeserializer for ResponseDeserializer {
    fn deserialize_structured<R: Sized, V: StructuredSerializer<R>>(self, visitor: V) -> Result<R> {
        if self.encoding() != Encoding::STRUCTURED {
            return Err(message::Error::WrongEncoding {});
        }
        visitor.set_structured_event(self.body.to_vec())
    }
}

impl<'a> MessageDeserializer for ResponseDeserializer {
    fn encoding(&self) -> Encoding {
        let contentheader = match self.headers.get("content-type") {
            Some(s) => s.as_str(),
            None => "",
        };
        if contentheader.starts_with("application/cloudevents+json") {
            Encoding::STRUCTURED
        } else if self
            .headers
            .get(super::headers::SPEC_VERSION_HEADER.as_str())
            .is_some()
        {
            Encoding::BINARY
        } else {
            Encoding::UNKNOWN
        }
    }
}

/// Method to transform an incoming [`Response`] to [`Event`].
pub async fn response_to_event(
    headers: HashMap<String, String>,
    body: Vec<u8>,
) -> std::result::Result<Event, surf::Error> {
    let mut bytes = BytesMut::with_capacity(body.len());
    bytes.extend_from_slice(body.as_slice());
    MessageDeserializer::into_event(ResponseDeserializer::new(headers, bytes.freeze()))
        .map_err(|e| Error::new(400, e))
}

/// Extention Trait for [`Response`] which acts as a wrapper for the function [`Response_to_event()`].
///
/// This trait is sealed and cannot be implemented for types outside of this crate.
#[allow(patterns_in_fns_without_body)]
#[async_trait]
pub trait ResponseExt: private::Sealed {
    /// Convert this [`Response`] into an [`Event`].
    async fn to_event(mut self) -> std::result::Result<Event, surf::Error>;
}

#[async_trait]
impl ResponseExt for Response {
    async fn to_event(mut self) -> std::result::Result<Event, surf::Error> {
        let mut headers = HashMap::new();
        for (n, v) in self.iter() {
            headers.insert(String::from(n.as_str()), String::from(v.as_str()));
        }
        let body = self.body_bytes().await?;
        response_to_event(headers, body).await
    }
}

mod private {
    // Sealing the ResponseExt
    pub trait Sealed {}
    impl Sealed for surf::Response {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use cloudevents::{EventBuilder, EventBuilderV10};
    use mockito::{mock};
    use serde_json::json;
    

    #[async_std::test]
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
            .time(time)
            .source("http://localhost")
            .extension("someint", "10")
            .build()
            .unwrap();

        let client = surf::Client::new();
        let res = client.get(&url).send().await.unwrap();
        let evt = res.to_event().await.unwrap();

        assert_eq!(expected, evt);
    }

    #[async_std::test]
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

            let client = surf::Client::new();
            let res = client.get(&url).send().await.unwrap();
            let evt = res.to_event().await.unwrap();

        assert_eq!(expected, evt);
    }

    #[async_std::test]
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

            let client = surf::Client::new();
            let res = client.get(&url).send().await.unwrap();
            let evt = res.to_event().await.unwrap();

        assert_eq!(expected, evt);
    }
}
