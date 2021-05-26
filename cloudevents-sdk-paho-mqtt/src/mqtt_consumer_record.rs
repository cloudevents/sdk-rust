use crate::headers;
use cloudevents::event::SpecVersion;
use cloudevents::message::{
    BinaryDeserializer, BinarySerializer, Encoding, MessageAttributeValue, MessageDeserializer,
    Result, StructuredDeserializer, StructuredSerializer,
};
use cloudevents::{message, Event};
use paho_mqtt::{Message, Properties, PropertyCode};
use std::convert::TryFrom;

pub struct ConsumerMessageDeserializer<'a> {
    pub(crate) headers: &'a Properties,
    pub(crate) payload: Option<Vec<u8>>,
}

impl<'a> ConsumerMessageDeserializer<'a> {
    fn get_mqtt_headers(message: &Message) -> &Properties {
        message.properties()
    }

    pub fn new(message: &Message) -> Result<ConsumerMessageDeserializer> {
        Ok(ConsumerMessageDeserializer {
            headers: Self::get_mqtt_headers(message),
            payload: Some(message.payload()).map(|s| Vec::from(s)),
        })
    }
}

impl<'a> BinaryDeserializer for ConsumerMessageDeserializer<'a> {
    fn deserialize_binary<R: Sized, V: BinarySerializer<R>>(self, mut visitor: V) -> Result<R> {
        if self.encoding() != Encoding::BINARY {
            return Err(message::Error::WrongEncoding {});
        }

        let spec_version = SpecVersion::try_from(
            self.headers
                .find_user_property(headers::SPEC_VERSION_HEADER)
                .unwrap()
                .as_str(),
        )?;

        visitor = visitor.set_spec_version(spec_version.clone())?;

        let attributes = spec_version.attribute_names();

        if let Some(hv) = self.headers.find_user_property(headers::CONTENT_TYPE) {
            visitor = visitor.set_attribute("datacontenttype", MessageAttributeValue::String(hv))?
        }

        for (hn, hv) in self
            .headers
            .user_iter()
            .filter(|(hn, _)| headers::SPEC_VERSION_HEADER != *hn && hn.starts_with("ce_"))
        {
            let name = &hn["ce_".len()..];

            if attributes.contains(&name) {
                visitor = visitor.set_attribute(name, MessageAttributeValue::String(hv))?
            } else {
                visitor = visitor.set_extension(name, MessageAttributeValue::String(hv))?
            }
        }

        if self.payload != None {
            visitor.end_with_data(self.payload.unwrap())
        } else {
            visitor.end()
        }
    }
}

impl<'a> StructuredDeserializer for ConsumerMessageDeserializer<'a> {
    fn deserialize_structured<R: Sized, V: StructuredSerializer<R>>(self, visitor: V) -> Result<R> {
        visitor.set_structured_event(self.payload.unwrap())
    }
}

impl<'a> MessageDeserializer for ConsumerMessageDeserializer<'a> {
    fn encoding(&self) -> Encoding {
        match self.headers.iter(PropertyCode::UserProperty).count() == 0 {
            true => Encoding::STRUCTURED,
            false => Encoding::BINARY,
        }
    }
}

pub fn record_to_event(msg: &Message) -> Result<Event> {
    MessageDeserializer::into_event(ConsumerMessageDeserializer::new(msg)?)
}

pub trait MessageExt {
    fn to_event(&self) -> Result<Event>;
}

impl MessageExt for Message {
    fn to_event(&self) -> Result<Event> {
        record_to_event(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::headers::MqttVersion::{MQTT_3, MQTT_5};
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

        let event = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .time(time)
            .source("http://localhost")
            .extension("someint", "10")
            .data("application/json", json!({"hello": "world"}))
            .build()
            .unwrap();

        let msg = MessageBuilder::new()
            .topic("test")
            .event(event, MQTT_5)
            .qos(1)
            .finalize();

        assert_eq!(msg.to_event().unwrap(), expected)
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

        let msg = MessageBuilder::new()
            .topic("test")
            .event(input, MQTT_3)
            .qos(1)
            .finalize();

        assert_eq!(msg.to_event().unwrap(), expected)
    }
}
