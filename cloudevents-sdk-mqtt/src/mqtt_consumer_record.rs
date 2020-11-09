use crate::headers;
use cloudevents::event::SpecVersion;
use cloudevents::message::{
    BinaryDeserializer, BinarySerializer, Encoding, MessageAttributeValue, MessageDeserializer,
    Result, StructuredDeserializer, StructuredSerializer,
};
use cloudevents::{message, Event};
use paho_mqtt::{Message, PropertyCode};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::str;

pub struct ConsumerMessageDeserializer {
    pub(crate) headers: HashMap<String, Vec<u8>>,
    pub(crate) payload: Option<Vec<u8>>,
}

impl ConsumerMessageDeserializer {
    fn get_mqtt_headers(message: &Message) -> Result<HashMap<String, Vec<u8>>> {
        let mut hm = HashMap::new();
        let prop_iterator = message.properties().iter(PropertyCode::UserProperty);

        for property in prop_iterator {
            let header = property.get_string_pair().unwrap();
            hm.insert(header.0.to_string(), Vec::from(header.1));
        }

        Ok(hm)
    }

    pub fn new(message: &Message) -> Result<ConsumerMessageDeserializer> {
        Ok(ConsumerMessageDeserializer {
            headers: Self::get_mqtt_headers(message)?,
            payload: Some(message.payload()).map(|s| Vec::from(s)),
        })
    }
}

impl BinaryDeserializer for ConsumerMessageDeserializer {
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

impl StructuredDeserializer for ConsumerMessageDeserializer {
    fn deserialize_structured<R: Sized, V: StructuredSerializer<R>>(self, visitor: V) -> Result<R> {
        visitor.set_structured_event(self.payload.unwrap())
    }
}

impl MessageDeserializer for ConsumerMessageDeserializer {
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

pub fn record_to_event(msg: &Message, version: headers::MqttVersion) -> Result<Event> {
    match version {
        headers::MqttVersion::V5 => {
            BinaryDeserializer::into_event(ConsumerMessageDeserializer::new(msg)?)
        }
        headers::MqttVersion::V3_1 => {
            StructuredDeserializer::into_event(ConsumerMessageDeserializer::new(msg)?)
        }
        headers::MqttVersion::V3_1_1 => {
            StructuredDeserializer::into_event(ConsumerMessageDeserializer::new(msg)?)
        }
    }
}

pub trait MessageExt {
    fn to_event(&self, version: headers::MqttVersion) -> Result<Event>;
}

impl MessageExt for Message {
    fn to_event(&self, version: headers::MqttVersion) -> Result<Event> {
        record_to_event(self, version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mqtt_producer_record::MessageRecord;

    use crate::MessageBuilderExt;
    use chrono::Utc;
    use cloudevents::event::Data;
    use cloudevents::{EventBuilder, EventBuilderV10};
    use paho_mqtt::MessageBuilder;
    use serde_json::json;

    #[test]
    fn test_binary_record() {
        let time = Utc::now();

        let expected = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .time(time)
            .source("http://localhost")
            .data(
                "application/json",
                Data::Binary(String::from("{\"hello\":\"world\"}").into_bytes()),
            )
            .extension("someint", "10")
            .build()
            .unwrap();

        let message_record = MessageRecord::from_event(
            EventBuilderV10::new()
                .id("0001")
                .ty("example.test")
                .time(time)
                .source("http://localhost")
                .extension("someint", "10")
                .data("application/json", json!({"hello": "world"}))
                .build()
                .unwrap(),
            headers::MqttVersion::V5,
        )
        .unwrap();

        let msg = MessageBuilder::new()
            .topic("test")
            .message_record(&message_record)
            .qos(1)
            .finalize();

        assert_eq!(msg.to_event(headers::MqttVersion::V5).unwrap(), expected)
    }

    #[test]
    fn test_structured_record() {
        let j = json!({"hello": "world"});

        let expected = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .source("http://localhost")
            .data("application/cloudevents+json", j.clone())
            .extension("someint", "10")
            .build()
            .unwrap();

        let input = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .source("http://localhost")
            .data("application/cloudevents+json", j.clone())
            .extension("someint", "10")
            .build()
            .unwrap();

        let serialized_event =
            StructuredDeserializer::deserialize_structured(input, MessageRecord::new()).unwrap();

        let msg = MessageBuilder::new()
            .topic("test")
            .message_record(&serialized_event)
            .qos(1)
            .finalize();

        assert_eq!(
            msg.to_event(headers::MqttVersion::V3_1_1).unwrap(),
            expected
        )
    }
}
