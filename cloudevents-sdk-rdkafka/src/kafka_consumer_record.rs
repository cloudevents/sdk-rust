use crate::headers;
use cloudevents::event::SpecVersion;
use cloudevents::message::{
    BinaryDeserializer, BinarySerializer, Encoding, MessageAttributeValue, MessageDeserializer,
    Result, StructuredDeserializer, StructuredSerializer,
};
use cloudevents::{message, Event};
use rdkafka::message::{BorrowedMessage, Headers, Message, OwnedMessage};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::str;

pub struct ConsumerRecordDeserializer {
    pub(crate) headers: HashMap<String, Vec<u8>>,
    pub(crate) payload: Vec<u8>,
}

impl ConsumerRecordDeserializer {
    pub fn owned_new(message: OwnedMessage) -> ConsumerRecordDeserializer {
        let mut resp_des = ConsumerRecordDeserializer {
            headers: HashMap::new(),
            payload: Vec::new(),
        };
        let headers = message.headers().unwrap();
        for i in 0..headers.count() {
            let header = headers.get(i).unwrap();
            resp_des
                .headers
                .insert(header.0.to_string(), Vec::from(header.1));
        }

        match message.payload() {
            Some(s) => resp_des.payload = Vec::from(s),
            None => resp_des.payload = resp_des.payload,
        }

        resp_des
    }

    pub fn borrowed_new(message: &BorrowedMessage) -> ConsumerRecordDeserializer {
        let mut resp_des = ConsumerRecordDeserializer {
            headers: HashMap::new(),
            payload: Vec::new(),
        };
        let headers = message.headers().unwrap();
        for i in 0..headers.count() {
            let header = headers.get(i).unwrap();
            resp_des
                .headers
                .insert(header.0.to_string(), Vec::from(header.1));
        }

        match message.payload() {
            Some(s) => resp_des.payload = Vec::from(s),
            None => resp_des.payload = resp_des.payload,
        }

        resp_des
    }
}

impl BinaryDeserializer for ConsumerRecordDeserializer {
    fn deserialize_binary<R: Sized, V: BinarySerializer<R>>(mut self, mut visitor: V) -> Result<R> {
        if self.encoding() != Encoding::BINARY {
            return Err(message::Error::WrongEncoding {});
        }

        let spec_version = SpecVersion::try_from(
            str::from_utf8(&self.headers.remove(headers::SPEC_VERSION_HEADER).unwrap()[..])
                .map_err(|e| cloudevents::message::Error::Other {
                    source: Box::new(e),
                })?,
        )?;

        visitor = visitor.set_spec_version(spec_version.clone())?;

        let attributes = spec_version.attribute_names();

        if let Some(hv) = self.headers.remove(headers::CONTENT_TYPE) {
            visitor = visitor.set_attribute(
                "datacontenttype",
                MessageAttributeValue::String(String::from_utf8(hv).map_err(|e| {
                    cloudevents::message::Error::Other {
                        source: Box::new(e),
                    }
                })?),
            )?
        }

        for (hn, hv) in self
            .headers
            .into_iter()
            .filter(|(hn, _)| headers::SPEC_VERSION_HEADER != *hn && hn.starts_with("ce_"))
        {
            let name = &hn["ce_".len()..];

            if attributes.contains(&name) {
                visitor = visitor.set_attribute(
                    name,
                    MessageAttributeValue::String(String::from_utf8(hv).map_err(|e| {
                        cloudevents::message::Error::Other {
                            source: Box::new(e),
                        }
                    })?),
                )?
            } else {
                visitor = visitor.set_extension(
                    name,
                    MessageAttributeValue::String(String::from_utf8(hv).map_err(|e| {
                        cloudevents::message::Error::Other {
                            source: Box::new(e),
                        }
                    })?),
                )?
            }
        }

        if self.payload.len() != 0 {
            visitor.end_with_data(self.payload.to_vec())
        } else {
            visitor.end()
        }
    }
}

impl StructuredDeserializer for ConsumerRecordDeserializer {
    fn deserialize_structured<R: Sized, V: StructuredSerializer<R>>(self, visitor: V) -> Result<R> {
        if self.encoding() != Encoding::STRUCTURED {
            return Err(message::Error::WrongEncoding {});
        }
        visitor.set_structured_event(self.payload.to_vec())
    }
}

impl MessageDeserializer for ConsumerRecordDeserializer {
    fn encoding(&self) -> Encoding {
        match (
            self.headers
                .get("content-type")
                .map(|s| String::from_utf8(s.to_vec()).ok())
                .flatten()
                .map(|s| s.starts_with(headers::CLOUDEVENTS_JSON_HEADER))
                .unwrap_or(false),
            self.headers.get(headers::SPEC_VERSION_HEADER),
        ) {
            (true, _) => Encoding::STRUCTURED,
            (_, Some(_)) => Encoding::BINARY,
            _ => Encoding::UNKNOWN,
        }
    }
}

/// Method to transform an incoming [`Response`] to [`Event`]
pub fn owned_record_to_event(msg: OwnedMessage) -> Result<Event> {
    MessageDeserializer::into_event(ConsumerRecordDeserializer::owned_new(msg))
}

pub fn borrowed_record_to_event(msg: &BorrowedMessage) -> Result<Event> {
    MessageDeserializer::into_event(ConsumerRecordDeserializer::borrowed_new(msg))
}

pub trait BorrowedMessageExt {
    fn into_event(&self) -> Result<Event>;
}

impl BorrowedMessageExt for BorrowedMessage<'_> {
    fn into_event(&self) -> Result<Event> {
        borrowed_record_to_event(self)
    }
}

pub trait OwnedMessageExt {
    fn into_event(self) -> Result<Event>;
}

impl OwnedMessageExt for OwnedMessage {
    fn into_event(self) -> Result<Event> {
        owned_record_to_event(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kafka_producer_record::ProducerRecordSerializer;
    use crate::EventExt;

    use chrono::Utc;
    use cloudevents::{EventBuilder, EventBuilderV10};
    use serde_json::json;
    use std::str::FromStr;
    use url::Url;

    #[test]
    fn test_binary_record() {
        let time = Utc::now();

        let expected = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            //TODO this is required now because the message deserializer implictly set default values
            // As soon as this defaulting doesn't happen anymore, we can remove it (Issues #40/#41)
            .time(time)
            .source(Url::from_str("http://localhost").unwrap())
            .extension("someint", "10")
            .build()
            .unwrap();

        // Since there is neither a way provided by rust-rdkafka to convert FutureProducer back into
        // OwnedMessage or BorrowedMessage, nor is there a way to create a BorrowedMessage struct,
        // the test uses OwnedMessage instead, which consumes the message instead of borrowing it like
        // in the case of BorrowedMessage

        let serialized_event = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .time(time)
            .source(Url::from_str("http://localhost").unwrap())
            .extension("someint", "10")
            .build()
            .unwrap()
            .serialize_event()
            .unwrap();

        let owned_message = OwnedMessage::new(
            serialized_event.payload,
            Some(String::from("test key").into_bytes()),
            String::from("test topic"),
            rdkafka::message::Timestamp::NotAvailable,
            10,
            10,
            Some(serialized_event.headers),
        );

        assert_eq!(owned_message.into_event().unwrap(), expected)
    }

    #[test]
    fn test_structured_record() {
        let j = json!({"hello": "world"});

        let expected = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .source(Url::from_str("http://localhost").unwrap())
            .data("application/json", j.clone())
            .extension("someint", "10")
            .build()
            .unwrap();

        // Since there is neither a way provided by rust-rdkafka to convert FutureProducer back into
        // OwnedMessage or BorrowedMessage, nor is there a way to create a BorrowedMessage struct,
        // the test uses OwnedMessage instead, which consumes the message instead of borrowing it like
        // in the case of BorrowedMessage

        let input = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .source(Url::from_str("http://localhost").unwrap())
            .data("application/json", j.clone())
            .extension("someint", "10")
            .build()
            .unwrap();

        let serialized_event =
            StructuredDeserializer::deserialize_structured(input, ProducerRecordSerializer::new())
                .unwrap();

        let owned_message = OwnedMessage::new(
            serialized_event.payload,
            Some(String::from("test key").into_bytes()),
            String::from("test topic"),
            rdkafka::message::Timestamp::NotAvailable,
            10,
            10,
            Some(serialized_event.headers),
        );

        assert_eq!(owned_message.into_event().unwrap(), expected)
    }
}
