use rdkafka_lib as rdkafka;

use crate::binding::{kafka::SPEC_VERSION_HEADER, CLOUDEVENTS_JSON_HEADER, CONTENT_TYPE};
use crate::event::SpecVersion;
use crate::message::{
    BinaryDeserializer, BinarySerializer, Encoding, MessageAttributeValue, MessageDeserializer,
    Result, StructuredDeserializer, StructuredSerializer,
};
use crate::{message, Event};
use rdkafka::message::{BorrowedMessage, Headers, Message, OwnedMessage};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::str;

/// Wrapper for [`Message`] that implements [`MessageDeserializer`] trait.
pub struct ConsumerRecordDeserializer {
    pub(crate) headers: HashMap<String, Vec<u8>>,
    pub(crate) payload: Option<Vec<u8>>,
}

impl ConsumerRecordDeserializer {
    fn get_kafka_headers(message: &impl Message) -> Result<HashMap<String, Vec<u8>>> {
        let mut hm = HashMap::new();
        let headers = message
            .headers()
            // TODO create an error variant for invalid headers
            .ok_or(crate::message::Error::WrongEncoding {})?;
        for i in 0..headers.count() {
            let header = headers.get(i).unwrap();
            hm.insert(header.0.to_string(), Vec::from(header.1));
        }
        Ok(hm)
    }

    pub fn new(message: &impl Message) -> Result<ConsumerRecordDeserializer> {
        Ok(ConsumerRecordDeserializer {
            headers: Self::get_kafka_headers(message)?,
            payload: message.payload().map(Vec::from),
        })
    }
}

impl BinaryDeserializer for ConsumerRecordDeserializer {
    fn deserialize_binary<R: Sized, V: BinarySerializer<R>>(mut self, mut visitor: V) -> Result<R> {
        if self.encoding() != Encoding::BINARY {
            return Err(message::Error::WrongEncoding {});
        }

        let spec_version = SpecVersion::try_from(
            str::from_utf8(&self.headers.remove(SPEC_VERSION_HEADER).unwrap()).map_err(|e| {
                crate::message::Error::Other {
                    source: Box::new(e),
                }
            })?,
        )?;

        visitor = visitor.set_spec_version(spec_version.clone())?;

        let attributes = spec_version.attribute_names();

        if let Some(hv) = self.headers.remove(CONTENT_TYPE) {
            visitor = visitor.set_attribute(
                "datacontenttype",
                MessageAttributeValue::String(String::from_utf8(hv).map_err(|e| {
                    crate::message::Error::Other {
                        source: Box::new(e),
                    }
                })?),
            )?
        }

        for (hn, hv) in self
            .headers
            .into_iter()
            .filter(|(hn, _)| SPEC_VERSION_HEADER != *hn && hn.starts_with("ce_"))
        {
            let name = &hn["ce_".len()..];

            if attributes.contains(&name) {
                visitor = visitor.set_attribute(
                    name,
                    MessageAttributeValue::String(String::from_utf8(hv).map_err(|e| {
                        crate::message::Error::Other {
                            source: Box::new(e),
                        }
                    })?),
                )?
            } else {
                visitor = visitor.set_extension(
                    name,
                    MessageAttributeValue::String(String::from_utf8(hv).map_err(|e| {
                        crate::message::Error::Other {
                            source: Box::new(e),
                        }
                    })?),
                )?
            }
        }

        if self.payload != None {
            visitor.end_with_data(self.payload.unwrap())
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
        visitor.set_structured_event(self.payload.unwrap())
    }
}

impl MessageDeserializer for ConsumerRecordDeserializer {
    fn encoding(&self) -> Encoding {
        match (
            self.headers
                .get("content-type")
                .map(|s| String::from_utf8(s.to_vec()).ok())
                .flatten()
                .map(|s| s.starts_with(CLOUDEVENTS_JSON_HEADER))
                .unwrap_or(false),
            self.headers.get(SPEC_VERSION_HEADER),
        ) {
            (true, _) => Encoding::STRUCTURED,
            (_, Some(_)) => Encoding::BINARY,
            _ => Encoding::UNKNOWN,
        }
    }
}

/// Method to transform a [`Message`] to [`Event`].
pub fn record_to_event(msg: &impl Message) -> Result<Event> {
    MessageDeserializer::into_event(ConsumerRecordDeserializer::new(msg)?)
}

/// Extension Trait for [`Message`] which acts as a wrapper for the function [`record_to_event()`].
///
/// This trait is sealed and cannot be implemented for types outside of this crate.
pub trait MessageExt: private::Sealed {
    /// Generates [`Event`] from [`BorrowedMessage`].
    fn to_event(&self) -> Result<Event>;
}

impl MessageExt for BorrowedMessage<'_> {
    fn to_event(&self) -> Result<Event> {
        record_to_event(self)
    }
}

impl MessageExt for OwnedMessage {
    fn to_event(&self) -> Result<Event> {
        record_to_event(self)
    }
}

mod private {
    use rdkafka_lib as rdkafka;

    // Sealing the MessageExt
    pub trait Sealed {}
    impl Sealed for rdkafka::message::OwnedMessage {}
    impl Sealed for rdkafka::message::BorrowedMessage<'_> {}
}

#[cfg(test)]
mod tests {
    use rdkafka_lib as rdkafka;

    use super::*;
    use crate::binding::rdkafka::kafka_producer_record::MessageRecord;

    use crate::{EventBuilder, EventBuilderV10};
    use chrono::Utc;
    use serde_json::json;

    #[test]
    fn test_binary_record() {
        let time = Utc::now();

        let expected = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .time(time)
            .source("http://localhost")
            .extension("someint", "10")
            .build()
            .unwrap();

        // Since there is neither a way provided by rust-rdkafka to convert FutureProducer back into
        // OwnedMessage or BorrowedMessage, nor is there a way to create a BorrowedMessage struct,
        // the test uses OwnedMessage instead, which consumes the message instead of borrowing it like
        // in the case of BorrowedMessage

        let message_record = MessageRecord::from_event(
            EventBuilderV10::new()
                .id("0001")
                .ty("example.test")
                .time(time)
                .source("http://localhost")
                .extension("someint", "10")
                .build()
                .unwrap(),
        )
        .unwrap();

        let owned_message = OwnedMessage::new(
            message_record.payload,
            Some(String::from("test key").into_bytes()),
            String::from("test topic"),
            rdkafka::message::Timestamp::NotAvailable,
            10,
            10,
            Some(message_record.headers),
        );

        assert_eq!(owned_message.to_event().unwrap(), expected)
    }

    #[test]
    fn test_structured_record() {
        let j = json!({"hello": "world"});

        let expected = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .source("http://localhost")
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
            .source("http://localhost")
            .data("application/json", j)
            .extension("someint", "10")
            .build()
            .unwrap();

        let serialized_event =
            StructuredDeserializer::deserialize_structured(input, MessageRecord::new()).unwrap();

        let owned_message = OwnedMessage::new(
            serialized_event.payload,
            Some(String::from("test key").into_bytes()),
            String::from("test topic"),
            rdkafka::message::Timestamp::NotAvailable,
            10,
            10,
            Some(serialized_event.headers),
        );

        assert_eq!(owned_message.to_event().unwrap(), expected)
    }
}
