use super::{Headers, SPEC_VERSION_HEADER};
use crate::{
    binding::CLOUDEVENTS_JSON_HEADER,
    event::SpecVersion,
    header_value_to_str, message,
    message::{
        BinaryDeserializer, BinarySerializer, Encoding, MessageAttributeValue, MessageDeserializer,
        Result, StructuredDeserializer, StructuredSerializer,
    },
};
use std::convert::TryFrom;

pub struct Deserializer<'a, T: Headers<'a>> {
    headers: &'a T,
    body: Vec<u8>,
}

impl<'a, T: Headers<'a>> Deserializer<'a, T> {
    pub fn new(headers: &'a T, body: Vec<u8>) -> Deserializer<'a, T> {
        Deserializer { headers, body }
    }
}

impl<'a, T: Headers<'a>> BinaryDeserializer for Deserializer<'a, T> {
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

        let attributes = spec_version.attribute_names();

        visitor = visitor.set_spec_version(spec_version)?;

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

        if let Some(hv) = self.headers.get(http::header::CONTENT_TYPE) {
            visitor = visitor.set_attribute(
                "datacontenttype",
                MessageAttributeValue::String(String::from(header_value_to_str!(hv)?)),
            )?
        }

        if !self.body.is_empty() {
            visitor.end_with_data(self.body)
        } else {
            visitor.end()
        }
    }
}

impl<'a, T: Headers<'a>> StructuredDeserializer for Deserializer<'a, T> {
    fn deserialize_structured<R: Sized, V: StructuredSerializer<R>>(self, visitor: V) -> Result<R> {
        if self.encoding() != Encoding::STRUCTURED {
            return Err(message::Error::WrongEncoding {});
        }
        visitor.set_structured_event(self.body)
    }
}

impl<'a, T: Headers<'a>> MessageDeserializer for Deserializer<'a, T> {
    fn encoding(&self) -> Encoding {
        if self
            .headers
            .get(http::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .filter(|&v| v.starts_with(CLOUDEVENTS_JSON_HEADER))
            .is_some()
        {
            Encoding::STRUCTURED
        } else if self.headers.get(SPEC_VERSION_HEADER).is_some() {
            Encoding::BINARY
        } else {
            Encoding::UNKNOWN
        }
    }
}
