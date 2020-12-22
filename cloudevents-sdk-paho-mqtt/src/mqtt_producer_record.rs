use super::headers;
use crate::headers::MqttVersion::MQTT_5;
use cloudevents::event::SpecVersion;
use cloudevents::message::{
    BinaryDeserializer, BinarySerializer, Error, MessageAttributeValue, Result,
    StructuredDeserializer, StructuredSerializer,
};
use cloudevents::Event;
use paho_mqtt::{MessageBuilder, Properties, Property, PropertyCode};
use std::option::Option::Some;

pub struct MessageRecord {
    pub(crate) headers: Properties,
    pub(crate) payload: Option<Vec<u8>>,
}

impl MessageRecord {
    /// Create a new empty [`MessageRecord`]
    pub fn new() -> Self {
        MessageRecord {
            headers: Properties::new(),
            payload: None,
        }
    }

    pub fn from_event(event: Event, version: &headers::MqttVersion) -> Result<Self> {
        match version {
            headers::MqttVersion::MQTT_5 => {
                BinaryDeserializer::deserialize_binary(event, MessageRecord::new())
            }
            headers::MqttVersion::MQTT_3 => {
                StructuredDeserializer::deserialize_structured(event, MessageRecord::new())
            }
        }
    }
}

impl BinarySerializer<MessageRecord> for MessageRecord {
    fn set_spec_version(mut self, spec_version: SpecVersion) -> Result<Self> {
        match Property::new_string_pair(
            PropertyCode::UserProperty,
            headers::SPEC_VERSION_HEADER,
            spec_version.as_str(),
        ) {
            Ok(property) => match self.headers.push(property) {
                Err(e) => Err(Error::Other {
                    source: Box::new(e),
                }),
                _ => Ok(self),
            },
            _ => Err(Error::UnknownAttribute {
                name: headers::SPEC_VERSION_HEADER.to_string(),
            }),
        }
    }

    fn set_attribute(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        match Property::new_string_pair(
            PropertyCode::UserProperty,
            &headers::ATTRIBUTES_TO_MQTT_HEADERS
                .get(name)
                .ok_or(cloudevents::message::Error::UnknownAttribute {
                    name: String::from(name),
                })?
                .clone()[..],
            &value.to_string()[..],
        ) {
            Ok(property) => match self.headers.push(property) {
                Err(e) => Err(Error::Other {
                    source: Box::new(e),
                }),
                _ => Ok(self),
            },
            _ => Err(Error::UnknownAttribute {
                name: headers::SPEC_VERSION_HEADER.to_string(),
            }),
        }
    }

    fn set_extension(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        match Property::new_string_pair(
            PropertyCode::UserProperty,
            &attribute_name_to_header!(name)[..],
            &value.to_string()[..],
        ) {
            Ok(property) => match self.headers.push(property) {
                Err(e) => Err(Error::Other {
                    source: Box::new(e),
                }),
                _ => Ok(self),
            },
            _ => Err(Error::UnknownAttribute {
                name: headers::SPEC_VERSION_HEADER.to_string(),
            }),
        }
    }

    fn end_with_data(mut self, bytes: Vec<u8>) -> Result<Self> {
        self.payload = Some(bytes);

        Ok(self)
    }

    fn end(self) -> Result<MessageRecord> {
        Ok(self)
    }
}

impl StructuredSerializer<MessageRecord> for MessageRecord {
    fn set_structured_event(mut self, bytes: Vec<u8>) -> Result<MessageRecord> {
        match Property::new_string_pair(
            PropertyCode::UserProperty,
            headers::CONTENT_TYPE,
            headers::CLOUDEVENTS_JSON_HEADER,
        ) {
            Ok(property) => match self.headers.push(property) {
                _ => (),
            },
            _ => (),
        }
        self.payload = Some(bytes);

        Ok(self)
    }
}

pub trait MessageBuilderExt {
    fn event(self, event: Event, version: headers::MqttVersion) -> MessageBuilder;
}

impl MessageBuilderExt for MessageBuilder {
    fn event(mut self, event: Event, version: headers::MqttVersion) -> MessageBuilder {
        let message_record =
            MessageRecord::from_event(event, &version).expect("error while serializing the event");

        match version {
            MQTT_5 => {
                self = self.properties(message_record.headers.clone());
            }
            _ => (),
        }

        if let Some(s) = message_record.payload.as_ref() {
            self = self.payload(s.to_vec());
        }

        self
    }
}
