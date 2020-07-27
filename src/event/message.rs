use super::Data;
use super::Event;
use super::{Attributes, AttributesReader};
use crate::event::SpecVersion;
use crate::message::{
    BinaryDeserializer, BinarySerializer, MessageAttributeValue, Result, StructuredDeserializer,
    StructuredSerializer,
};
use crate::{EventBuilderV10, EventBuilderV03, EventBuilder};

impl StructuredDeserializer for Event {
    fn deserialize_structured<R, V: StructuredSerializer<R>>(self, visitor: V) -> Result<R> {
        let vec: Vec<u8> = serde_json::to_vec(&self)?;
        visitor.set_structured_event(vec)
    }
}

impl BinaryDeserializer for Event {
    fn deserialize_binary<R: Sized, V: BinarySerializer<R>>(self, mut visitor: V) -> Result<R> {
        visitor = visitor.set_spec_version(self.get_specversion())?;
        visitor = self.attributes.deserialize_attributes(visitor)?;
        for (k, v) in self.extensions.into_iter() {
            visitor = visitor.set_extension(&k, v.into())?;
        }
        match self.data {
            Some(Data::String(s)) => visitor.end_with_data(s.into_bytes()),
            Some(Data::Binary(v)) => visitor.end_with_data(v),
            Some(Data::Json(j)) => {
                let vec: Vec<u8> = serde_json::to_vec(&j)?;
                visitor.end_with_data(vec)
            }
            None => visitor.end(),
        }
    }
}

pub(crate) trait AttributesDeserializer {
    fn deserialize_attributes<R: Sized, V: BinarySerializer<R>>(self, visitor: V) -> Result<V>;
}

impl AttributesDeserializer for Attributes {
    fn deserialize_attributes<R: Sized, V: BinarySerializer<R>>(self, visitor: V) -> Result<V> {
        match self {
            Attributes::V03(v03) => v03.deserialize_attributes(visitor),
            Attributes::V10(v10) => v10.deserialize_attributes(visitor),
        }
    }
}

pub(crate) trait AttributesSerializer {
    fn serialize_attribute(&mut self, name: &str, value: MessageAttributeValue) -> Result<()>;
}

#[derive(Debug)]
pub(crate) struct EventStructuredSerializer {}

impl StructuredSerializer<Event> for EventStructuredSerializer {
    fn set_structured_event(self, bytes: Vec<u8>) -> Result<Event> {
        Ok(serde_json::from_slice(&bytes)?)
    }
}

#[derive(Debug)]
pub(crate) enum EventBinarySerializer {
    V10(EventBuilderV10),
    V03(EventBuilderV03)
}

impl EventBinarySerializer {
    pub(crate) fn new() -> Self {
        EventBinarySerializer::V10(EventBuilderV10::new())
    }
}

impl BinarySerializer<Event> for EventBinarySerializer {
    fn set_spec_version(self, spec_version: SpecVersion) -> Result<Self> {
        Ok(match spec_version {
            SpecVersion::V03 => EventBinarySerializer::V03(EventBuilderV03::new()),
            SpecVersion::V10 => EventBinarySerializer::V10(EventBuilderV10::new())
        })
    }

    fn set_attribute(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        match &mut self {
            EventBinarySerializer::V03(eb) => eb.serialize_attribute(name, value)?,
            EventBinarySerializer::V10(eb) => eb.serialize_attribute(name, value)?,
        }
        Ok(self)
    }

    fn set_extension(self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        Ok(match self {
            EventBinarySerializer::V03(eb) => EventBinarySerializer::V03(
                eb.extension(name, value)
            ),
            EventBinarySerializer::V10(eb) => EventBinarySerializer::V10(
                eb.extension(name, value)
            )
        })
    }

    fn end_with_data(self, bytes: Vec<u8>) -> Result<Event> {
        Ok(match self {
            EventBinarySerializer::V03(eb) => eb.data_without_content_type(Data::Binary(bytes)).build(),
            EventBinarySerializer::V10(eb) => eb.data_without_content_type(Data::Binary(bytes)).build()
        }?)
    }

    fn end(self) -> Result<Event> {
        Ok(match self {
            EventBinarySerializer::V03(eb) => eb.build(),
            EventBinarySerializer::V10(eb) => eb.build()
        }?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::Error;

    #[test]
    fn binary_deserializer_unrecognized_attribute_v03() {
        assert_eq!(
            Error::UnrecognizedAttributeName {
                name: "dataschema".to_string()
            }.to_string(),
            EventBinarySerializer::new()
                .set_spec_version(SpecVersion::V03).unwrap()
                .set_attribute("dataschema", MessageAttributeValue::Boolean(true))
                .expect_err("Should return an error")
                .to_string()
        )
    }

    #[test]
    fn binary_deserializer_missing_id() {
        assert_eq!(
            Error::EventBuilderError {
                source: crate::event::EventBuilderError::MissingRequiredAttribute {
                    attribute_name: "id"
                },
            }.to_string(),
            EventBinarySerializer::new()
                .set_spec_version(SpecVersion::V10).unwrap()
                .end()
                .unwrap_err()
                .to_string()
        )
    }

    #[test]
    fn binary_deserializer_unrecognized_attribute_v10() {
        assert_eq!(
            Error::UnrecognizedAttributeName {
                name: "schemaurl".to_string()
            }.to_string(),
            EventBinarySerializer::new()
                .set_spec_version(SpecVersion::V10).unwrap()
                .set_attribute("schemaurl", MessageAttributeValue::Boolean(true))
                .expect_err("Should return an error")
                .to_string()
        )
    }
}