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
    pub(crate) payload: Option<Vec<u8>>,
}

impl ConsumerRecordDeserializer {
    fn get_kafka_headers(message: &impl Message) -> Result<HashMap<String, Vec<u8>>> {
        let mut hm = HashMap::new();
        let headers = message
            .headers()
            // TODO create an error variant for invalid headers
            .ok_or(cloudevents::message::Error::WrongEncoding {})?;
        for i in 0..headers.count() {
            let header = headers.get(i).unwrap();
            hm.insert(header.0.to_string(), Vec::from(header.1));
        }
        Ok(hm)
    }

    pub fn new(message: &impl Message) -> Result<ConsumerRecordDeserializer> {
        Ok(ConsumerRecordDeserializer {
            headers: Self::get_kafka_headers(message)?,
            payload: message.payload().map(|s| Vec::from(s)),
        })
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
pub fn record_to_event(msg: &impl Message) -> Result<Event> {
    MessageDeserializer::into_event(ConsumerRecordDeserializer::new(msg)?)
}

pub trait BorrowedMessageExt {
    fn event_from(&self) -> Result<Event>;
}

impl BorrowedMessageExt for BorrowedMessage<'_> {
    fn event_from(&self) -> Result<Event> {
        record_to_event(self)
    }
}

pub trait OwnedMessageExt {
    fn event_from(&self) -> Result<Event>;
}

impl OwnedMessageExt for OwnedMessage {
    fn event_from(&self) -> Result<Event> {
        record_to_event(self)
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

        assert_eq!(owned_message.event_from().unwrap(), expected)
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

        assert_eq!(owned_message.event_from().unwrap(), expected)
    }
}
