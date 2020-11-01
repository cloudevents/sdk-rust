use super::headers;
use bytes::Bytes;
use http::{header::HeaderName, HeaderMap};

use cloudevents::event::SpecVersion;
use cloudevents::message::{
    BinaryDeserializer, BinarySerializer, Encoding, Error, MessageAttributeValue,
    MessageDeserializer, Result, StructuredDeserializer, StructuredSerializer,
};

use cloudevents::{message, Event};
use std::convert::TryFrom;

pub struct RequestDeserializer {
    headers: HeaderMap,
    body: Bytes,
}

impl RequestDeserializer {
    pub fn new(headers: HeaderMap, body: Bytes) -> RequestDeserializer {
        RequestDeserializer { headers, body }
    }
}

impl BinaryDeserializer for RequestDeserializer {
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

impl StructuredDeserializer for RequestDeserializer {
    fn deserialize_structured<R: Sized, V: StructuredSerializer<R>>(self, visitor: V) -> Result<R> {
        if self.encoding() != Encoding::STRUCTURED {
            return Err(message::Error::WrongEncoding {});
        }
        visitor.set_structured_event(self.body.to_vec())
    }
}

impl MessageDeserializer for RequestDeserializer {
    fn encoding(&self) -> Encoding {
        if self
            .headers
            .get("content-type")
            .map(|v| v.to_str().unwrap_or(""))
            .unwrap_or("")
            == "application/cloudevents+json"
        {
            Encoding::STRUCTURED
        } else if self
            .headers
            .get::<&'static HeaderName>(&super::headers::SPEC_VERSION_HEADER)
            .is_some()
        {
            Encoding::BINARY
        } else {
            Encoding::UNKNOWN
        }
    }
}

pub fn request_to_event(req: HeaderMap, bytes: bytes::Bytes) -> std::result::Result<Event, Error> {
    MessageDeserializer::into_event(RequestDeserializer::new(req, bytes))
}
