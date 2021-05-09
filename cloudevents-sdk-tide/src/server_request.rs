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
use tide::{Error, Request};

/// Wrapper for [`Request`] that implements [`MessageDeserializer`] trait.
pub struct RequestDeserializer {
    headers: HashMap<String, String>,
    body: Bytes,
}

impl RequestDeserializer {
    pub fn new(headers: HashMap<String, String>, body: Bytes) -> RequestDeserializer {
        RequestDeserializer { headers, body }
    }
}

impl<'a> BinaryDeserializer for RequestDeserializer {
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

        if let Some(hv) = self.headers.get("content-type") {
            visitor = visitor.set_attribute(
                "datacontenttype",
                MessageAttributeValue::String(String::from(hv.as_str())),
            )?
        }

        if !self.body.is_empty() {
            visitor.end_with_data(self.body.to_vec())
        } else {
            visitor.end()
        }
    }
}

impl<'a> StructuredDeserializer for RequestDeserializer {
    fn deserialize_structured<R: Sized, V: StructuredSerializer<R>>(self, visitor: V) -> Result<R> {
        if self.encoding() != Encoding::STRUCTURED {
            return Err(message::Error::WrongEncoding {});
        }
        visitor.set_structured_event(self.body.to_vec())
    }
}

impl<'a> MessageDeserializer for RequestDeserializer {
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

/// Method to transform an incoming [`Request`] to [`Event`].
pub async fn request_to_event(
    headers: HashMap<String, String>,
    body: Vec<u8>,
) -> std::result::Result<Event, tide::Error> {
    let mut bytes = BytesMut::with_capacity(body.len());
    bytes.extend_from_slice(body.as_slice());
    MessageDeserializer::into_event(RequestDeserializer::new(headers, bytes.freeze()))
        .map_err(|e| Error::new(400, e))
}

/// Extention Trait for [`Request`] which acts as a wrapper for the function [`request_to_event()`].
///
/// This trait is sealed and cannot be implemented for types outside of this crate.
#[allow(patterns_in_fns_without_body)]
#[async_trait]
pub trait RequestExt: private::Sealed {
    /// Convert this [`Request`] into an [`Event`].
    async fn to_event(&self, mut body: Vec<u8>) -> std::result::Result<Event, tide::Error>;
}

#[async_trait]
impl<State: Clone + Send + Sync + 'static> RequestExt for Request<State> {
    async fn to_event(&self, body: Vec<u8>) -> std::result::Result<Event, tide::Error> {
        let mut headers = HashMap::new();
        for (n, v) in self.iter() {
            headers.insert(String::from(n.as_str()), String::from(v.as_str()));
        }
        request_to_event(headers, body).await
    }
}

mod private {
    // Sealing the RequestExt
    pub trait Sealed {}
    impl<State> Sealed for tide::Request<State> {}
}
// : Unpin + Clone + Send + Sync + 'static
#[cfg(test)]
mod tests {
    use super::*;
    // use chrono::Utc;
    use cloudevents::{EventBuilder, EventBuilderV10};
    use serde_json::{json, Value};
    use tide::{Body, Request};
    use tide_testing::TideTestingExt;
    use chrono::Utc;

    #[async_std::test]
    async fn test_request() {
        let time = Utc::now();
        let mut app = tide::new();
        app.at("/").post(move |mut req: Request<()>| async move {
            let expected = EventBuilderV10::new()
                .id("0001")
                .ty("example.test")
                .source("http://localhost/")
                .time(time)
                .data(
                    "application/octet-stream",
                    String::from("hello").into_bytes(),
                )
                .build()
                .unwrap();

            let body = req.body_bytes().await.unwrap();
            let evtresp: Event = req.to_event(body).await.unwrap();

            assert_eq!(expected, evtresp);
            Ok(Body::from_json(&evtresp)?)
        });

        match app
            .post("/")
            .body(tide::Body::from_string("hello".into()))
            .content_type("application/octet-stream")
            .header("ce-specversion", "1.0")
            .header("ce-id", "0001")
            .header("ce-type", "example.test")
            .header("ce-source", "http://localhost/")
            .header("ce-time", time.to_rfc3339())
            .recv_string()
            .await
        {
            Ok(r) => {
                println!("{}", r);
                r
            }
            Err(e) => panic!("Get String Failed {:?}", e),
        };
    }

    #[async_std::test]
    async fn test_request_with_full_data() {
        let mut app = tide::new();
        app.at("/").post(|mut req: Request<()>| async move {
            let body = req.body_bytes().await.unwrap();
            let expected = EventBuilderV10::new()
                .id("0001")
                .ty("example.test")
                .source("http://localhost/")
                //TODO this is required now because the message deserializer implictly set default values
                // As soon as this defaulting doesn't happen anymore, we can remove it (Issues #40/#41)
                .data(
                    "application/json",
                    r#"{"hello":"world"}"#.as_bytes().to_vec(),
                )
                .extension("someint", "10")
                .build()
                .unwrap();

            let evtresp: Event = req.to_event(body.to_vec()).await.unwrap();
            assert_eq!(expected, evtresp);
            Ok(Body::from_json(&evtresp)?)
        });

        match app
            .post("/")
            .body(tide::Body::from_string(r#"{"hello":"world"}"#.into()))
            .content_type("application/json")
            .header("ce-specversion", "1.0")
            .header("ce-id", "0001")
            .header("ce-type", "example.test")
            .header("ce-source", "http://localhost/")
            .header("ce-someint", "10")
            .recv_string()
            .await
        {
            Ok(r) => {
                println!("{}", r);
                r
            }
            Err(e) => panic!("Get String Failed {:?}", e),
        };
    }
    #[derive(Clone)]
    struct State {}
    #[async_std::test]
    async fn test_request_with_cloudevent() {
        let state = State {};
        let mut app = tide::with_state(state);
        app.at("/").post(|mut req: Request<State>| async move {
            let body = req.body_string().await.unwrap();
            let expecteddata = json!({ "hello":"world" });
            let j: Value = serde_json::from_str(&body).unwrap();
            let vec: Vec<u8> = serde_json::to_vec(&j).unwrap();
            let expected = EventBuilderV10::new()
                .id("0001")
                .ty("example.test")
                .source("http://localhost/")
                .data("application/cloudevents+json", expecteddata)
                .extension("someint", "10")
                .build()
                .unwrap();

            let evtresp: Event = req.to_event(vec).await.unwrap();
            assert_eq!(expected, evtresp);
            Ok(Body::from_json(&evtresp)?)
        });

        match app
            .post("/")
            .body(tide::Body::from_string(
                r#"{
                "datacontenttype" : "application/cloudevents+json",
                "data" : { "hello":"world" },
                "specversion" : "1.0",
                "id" : "0001",
                "type" : "example.test",
                "source" : "http://localhost/",
                "someint" : "10"
            }"#
                .into(),
            ))
            .content_type("application/cloudevents+json")
            .header("ce-specversion", "1.0")
            .header("ce-id", "0001")
            .header("ce-type", "example.test")
            .header("ce-source", "http://localhost/")
            .header("ce-someint", "10")
            .recv_string()
            .await
        {
            Ok(r) => {
                println!("{}", r);
                r
            }
            Err(e) => panic!("Get String Failed {:?}", e),
        };
    }
}
