use super::Data;
use super::Event;
use super::{Attributes, AttributesReader};
use crate::event::SpecVersion;
use crate::message::{
    BinaryDeserializer, BinarySerializer, DeserializationResult, Error, MessageAttributeValue,
    SerializationResult, StructuredDeserializer, StructuredSerializer,
};

impl StructuredDeserializer for Event {
    fn deserialize_structured<R, V: StructuredSerializer<R>>(self, visitor: V) -> Result<R, Error> {
        let vec: Vec<u8> = serde_json::to_vec(&self)?;
        visitor.set_structured_event(vec)
    }
}

impl BinaryDeserializer for Event {
    fn deserialize_binary<R: Sized, V: BinarySerializer<R>>(
        self,
        mut visitor: V,
    ) -> Result<R, Error> {
        visitor.set_spec_version(self.get_specversion())?;
        self.attributes.deserialize_attributes(&mut visitor)?;
        for (k, v) in self.extensions.into_iter() {
            visitor.set_extension(&k, v.into())?;
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
    fn deserialize_attributes<R: Sized, V: BinarySerializer<R>>(
        self,
        visitor: &mut V,
    ) -> DeserializationResult;
}

pub(crate) trait AttributesSerializer {
    fn serialize_attribute(
        &mut self,
        name: &str,
        value: MessageAttributeValue,
    ) -> SerializationResult;
}

impl AttributesDeserializer for Attributes {
    fn deserialize_attributes<R: Sized, V: BinarySerializer<R>>(
        self,
        visitor: &mut V,
    ) -> DeserializationResult {
        match self {
            Attributes::V03(v03) => v03.deserialize_attributes(visitor),
            Attributes::V10(v10) => v10.deserialize_attributes(visitor),
        }
    }
}

impl AttributesSerializer for Attributes {
    fn serialize_attribute(
        &mut self,
        name: &str,
        value: MessageAttributeValue,
    ) -> SerializationResult {
        match self {
            Attributes::V03(v03) => v03.serialize_attribute(name, value),
            Attributes::V10(v10) => v10.serialize_attribute(name, value),
        }
    }
}

impl StructuredSerializer<Event> for Event {
    fn set_structured_event(mut self, bytes: Vec<u8>) -> Result<Event, Error> {
        let new_event: Event = serde_json::from_slice(&bytes)?;
        self.attributes = new_event.attributes;
        self.data = new_event.data;
        self.extensions = new_event.extensions;
        Ok(self)
    }
}

impl BinarySerializer<Event> for Event {
    fn set_spec_version(&mut self, spec_version: SpecVersion) -> SerializationResult {
        match spec_version {
            SpecVersion::V03 => self.attributes = self.attributes.clone().into_v03(),
            SpecVersion::V10 => self.attributes = self.attributes.clone().into_v10(),
        }
        Ok(())
    }

    fn set_attribute(&mut self, name: &str, value: MessageAttributeValue) -> SerializationResult {
        self.attributes.serialize_attribute(name, value)
    }

    fn set_extension(&mut self, name: &str, value: MessageAttributeValue) -> SerializationResult {
        self.extensions.insert(name.to_string(), value.into());
        Ok(())
    }

    fn end_with_data(mut self, bytes: Vec<u8>) -> Result<Event, Error> {
        self.data = Some(Data::from_binary(self.get_datacontenttype(), bytes)?);
        Ok(self)
    }

    fn end(self) -> Result<Event, Error> {
        Ok(self)
    }
}
