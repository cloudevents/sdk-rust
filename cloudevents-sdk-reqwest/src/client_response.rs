use super::headers;
use bytes::Bytes;
use cloudevents::event::SpecVersion;
use cloudevents::message::{
    BinaryDeserializer, BinarySerializer, Encoding, Error, MessageAttributeValue,
    MessageDeserializer, Result, StructuredDeserializer, StructuredSerializer,
};
use cloudevents::{message, Event};
use reqwest::header::{HeaderMap, HeaderName};
use reqwest::Response;
use std::convert::TryFrom;

/// Wrapper for [`Response`] that implements [`MessageDeserializer`] trait
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
            unwrap_optional_header!(self.headers, headers::SPEC_VERSION_HEADER).unwrap()?,
        )?;

        visitor = visitor.set_spec_version(spec_version.clone())?;

        let attributes = spec_version.attribute_names();

        for (hn, hv) in self
            .headers
            .iter()
            .filter(|(hn, _)| headers::SPEC_VERSION_HEADER.ne(hn) && hn.as_str().starts_with("ce-"))
        {
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

        if self.body.len() != 0 {
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
            unwrap_optional_header!(self.headers, reqwest::header::CONTENT_TYPE)
                .map(|r| r.ok())
                .flatten()
                .map(|e| e.starts_with("application/cloudevents+json")),
            self.headers
                .get::<&'static HeaderName>(&headers::SPEC_VERSION_HEADER),
        ) {
            (Some(true), _) => Encoding::STRUCTURED,
            (_, Some(_)) => Encoding::BINARY,
            _ => Encoding::UNKNOWN,
        }
    }
}

/// Method to transform an incoming [`Response`] to [`Event`]
pub async fn response_to_event(res: Response) -> Result<Event> {
    let h = res.headers().to_owned();
    let b = res.bytes().await.map_err(|e| Error::Other {
        source: Box::new(e),
    })?;

    MessageDeserializer::into_event(ResponseDeserializer::new(h, b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::mock;

    use cloudevents::{EventBuilder, EventBuilderV10};
    use serde_json::json;
    use std::str::FromStr;
    use url::Url;

    #[tokio::test]
    async fn test_response() {
        let url = mockito::server_url();
        let _m = mock("GET", "/")
            .with_status(200)
            .with_header("ce-specversion", "1.0")
            .with_header("ce-id", "0001")
            .with_header("ce-type", "example.test")
            .with_header("ce-source", "http://localhost")
            .with_header("ce-someint", "10")
            .create();

        let expected = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .source(Url::from_str("http://localhost").unwrap())
            .extension("someint", "10")
            .build()
            .unwrap();

        let client = reqwest::Client::new();
        let res = client.get(&url).send().await.unwrap();

        let resp = response_to_event(res).await.unwrap();
        assert_eq!(expected, resp);
    }

    #[tokio::test]
    async fn test_response_with_full_data() {
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
            .with_body(j.to_string())
            .create();

        let expected = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .source(Url::from_str("http://localhost").unwrap())
            .data("application/json", j.clone())
            .extension("someint", "10")
            .build()
            .unwrap();

        let client = reqwest::Client::new();
        let res = client.get(&url).send().await.unwrap();

        let resp = response_to_event(res).await.unwrap();
        assert_eq!(expected, resp);
    }

    #[tokio::test]
    async fn test_structured_response_with_full_data() {
        let j = json!({"hello": "world"});
        let expected = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .source(Url::from_str("http://localhost").unwrap())
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
        let res = client.get(&url).send().await.unwrap();

        let resp = response_to_event(res).await.unwrap();
        assert_eq!(expected, resp);
    }
}
