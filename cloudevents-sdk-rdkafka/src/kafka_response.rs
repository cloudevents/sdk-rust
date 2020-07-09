#[macro_use]
use async_trait::async_trait;
use bytes::Bytes;
use cloudevents::event::SpecVersion;
use cloudevents::message::{
    BinaryDeserializer, BinarySerializer, Encoding, MessageAttributeValue, MessageDeserializer,
    Result, StructuredDeserializer, StructuredSerializer,
};
use cloudevents::{message, Event};
use rdkafka::message::{Headers, Message, OwnedMessage};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::str;

pub struct ResponseDeserializer {
    headers: HashMap<String, Bytes>,
    payload: Bytes,
}

impl ResponseDeserializer {
    pub fn new(message: &OwnedMessage) -> ResponseDeserializer {
        let mut resp_des = ResponseDeserializer {
            headers: HashMap::new(),
            payload: Bytes::new(),
        };
        let headers = message.headers().unwrap();
        for i in 0..headers.count() {
            let header = headers.get(i).unwrap();
            resp_des
                .headers
                .insert(header.0.to_string(), Bytes::copy_from_slice(header.1));
        }

        resp_des.payload = Bytes::copy_from_slice(message.payload().unwrap());

        resp_des
    }
}

impl BinaryDeserializer for ResponseDeserializer {
    fn deserialize_binary<R: Sized, V: BinarySerializer<R>>(self, mut visitor: V) -> Result<R> {
        if self.encoding() != Encoding::BINARY {
            return Err(message::Error::WrongEncoding {});
        }

        let spec_version = SpecVersion::try_from(header_value_to_str!(self
            .headers
            .get("ce_specversion")
            .unwrap())?)?;

        visitor = visitor.set_spec_version(spec_version.clone())?;

        let attributes = spec_version.attribute_names();

        for (hn, hv) in self
            .headers
            .iter()
            .filter(|(hn, _)| "ce_specversion" != **hn && hn.starts_with("ce-"))
        {
            let name = &hn["ce_".len()..];

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

        if self.payload.len() != 0 {
            visitor.end_with_data(self.payload.to_vec())
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
        visitor.set_structured_event(self.payload.to_vec())
    }
}

impl MessageDeserializer for ResponseDeserializer {
    fn encoding(&self) -> Encoding {
        match (
            str::from_utf8(self.headers.get("content-type").unwrap())
                .unwrap_or("UNKNOWN")
                .starts_with("application/cloudevents+json"),
            self.headers.get("ce_specversion"),
        ) {
            (true, _) => Encoding::STRUCTURED,
            (_, Some(_)) => Encoding::BINARY,
            _ => Encoding::UNKNOWN,
        }
    }
}

/// Method to transform an incoming [`Response`] to [`Event`]
pub async fn response_to_event(res: OwnedMessage) -> Result<Event> {
    MessageDeserializer::into_event(ResponseDeserializer::new(&res))
}

#[async_trait(?Send)]
pub trait OwnedMessageExt {
    async fn into_event(self) -> Result<Event>;
}

#[async_trait(?Send)]
impl OwnedMessageExt for OwnedMessage {
    async fn into_event(self) -> Result<Event> {
        response_to_event(self).await
    }
}
