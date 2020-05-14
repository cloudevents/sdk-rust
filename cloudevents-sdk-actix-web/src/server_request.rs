use actix_web::http::HeaderName;
use actix_web::{HttpMessage, HttpRequest, web};
use cloudevents::message::{BinaryDeserializer, BinarySerializer, Encoding, MessageDeserializer, StructuredDeserializer, StructuredSerializer, MessageAttributeValue, Error};
use cloudevents::{Event, message};
use actix_web::web::{BytesMut, Bytes};
use futures::StreamExt;
use bytes::buf::BufExt;
use cloudevents::event::SpecVersion;
use std::convert::TryFrom;
use super::headers;

/// Wrapper for [`HttpRequest`] that implements [`MessageDeserializer`] trait
pub struct HttpRequestMessage<'a> {
    req: &'a HttpRequest,
    body_reader: Bytes
}

impl HttpRequestMessage<'_> {
    fn new(req: &HttpRequest, body_reader: Bytes) -> HttpRequestMessage {
        HttpRequestMessage {
            req, body_reader
        }
    }
}

impl<'a> BinaryDeserializer for HttpRequestMessage<'a> {
    fn deserialize_binary<R: Sized, V: BinarySerializer<R>>(self, mut visitor: V) -> Result<R, Error> {
        if self.encoding() != Encoding::BINARY {
            return Err(message::Error::WrongEncoding {})
        }

        let spec_version = SpecVersion::try_from(
            unwrap_optional_header!(self.req.headers(), headers::SPEC_VERSION_HEADER).unwrap()?
        )?;

        visitor.set_spec_version(spec_version.clone())?;

        let attributes = cloudevents::event::spec_version::ATTRIBUTE_NAMES.get(&spec_version).unwrap();

        for (hn, hv) in self.req
            .headers()
            .iter()
            .filter(|(hn, _)| headers::SPEC_VERSION_HEADER.ne(hn) && hn.as_str().starts_with("ce-")) {
            let name = hn.as_str().strip_prefix("ce-").unwrap();

            if attributes.contains(&name) {
                visitor.set_attribute(name, MessageAttributeValue::String(String::from(
                    header_value_to_str!(hv)?
                )))?
            } else {
                visitor.set_extension(name, MessageAttributeValue::String(String::from(
                    header_value_to_str!(hv)?
                )))?
            }
        }

        if let Some(hv) = self.req.headers().get("content-type") {
            visitor.set_attribute("datacontenttype", MessageAttributeValue::String(String::from(
                header_value_to_str!(hv)?
            )))?
        }

        if self.body_reader.len() != 0 {
            visitor.end_with_data(self.body_reader.reader())
        } else {
            visitor.end()
        }
    }
}

impl<'a> StructuredDeserializer for HttpRequestMessage<'a> {
    fn deserialize_structured<R: Sized, V: StructuredSerializer<R>>(
        self,
        visitor: V,
    ) -> Result<R, Error> {
        if self.encoding() != Encoding::STRUCTURED {
            return Err(message::Error::WrongEncoding {})
        }
        visitor.set_structured_event(self.body_reader.reader())
    }
}

impl<'a> MessageDeserializer for HttpRequestMessage<'a> {
    fn encoding(&self) -> Encoding {
        if self.req.content_type() == "application/cloudevents+json" {
            Encoding::STRUCTURED
        } else if self
            .req
            .headers()
            .get::<&'static HeaderName>(&super::headers::SPEC_VERSION_HEADER)
            .is_some()
        {
            Encoding::BINARY
        } else {
            Encoding::UNKNOWN
        }
    }
}

/// Method to transform an incoming [`HttpRequest`] to [`Event`]
pub async fn request_to_event(req: &HttpRequest, mut payload: web::Payload) -> Result<Event, actix_web::error::Error> {
    let mut bytes = BytesMut::new();
    while let Some(item) = payload.next().await {
        bytes
            .extend_from_slice(
                &item?
            );
    }
    MessageDeserializer::into_event(HttpRequestMessage::new(req, bytes.freeze()))
        .map_err(actix_web::error::ErrorBadRequest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;
    use url::Url;

    use serde_json::json;
    use cloudevents::EventBuilder;
    use std::str::FromStr;

    #[actix_rt::test]
    async fn test_request() {
        let expected = EventBuilder::new()
            .id("0001")
            .ty("example.test")
            .source(Url::from_str("http://localhost").unwrap())
            .extension("someint", "10")
            .build();

        let (req, payload) = test::TestRequest::post()
            .header("ce-specversion", "1.0")
            .header("ce-id", "0001")
            .header("ce-type", "example.test")
            .header("ce-source", "http://localhost")
            .header("ce-someint", "10")
            .to_http_parts();

        let resp = request_to_event(&req, web::Payload(payload)).await.unwrap();
        assert_eq!(expected, resp);
    }

    #[actix_rt::test]
    async fn test_request_with_full_data() {
        let j = json!({"hello": "world"});

        let expected = EventBuilder::new()
            .id("0001")
            .ty("example.test")
            .source(Url::from_str("http://localhost").unwrap())
            .data("application/json", j.clone())
            .extension("someint", "10")
            .build();

        let (req, payload) = test::TestRequest::post()
            .header("ce-specversion", "1.0")
            .header("ce-id", "0001")
            .header("ce-type", "example.test")
            .header("ce-source", "http://localhost")
            .header("ce-someint", "10")
            .header("content-type", "application/json")
            .set_json(&j)
            .to_http_parts();

        let resp = request_to_event(&req, web::Payload(payload)).await.unwrap();
        assert_eq!(expected, resp);
    }

}