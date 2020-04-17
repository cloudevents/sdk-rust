use super::Data;
use super::Event;
use super::{Attributes, AttributesReader};
use crate::event::SpecVersion;
use crate::message::{
    BinaryDeserializer, BinaryVisitor, DeserializationResult, MessageAttributeValue,
    SerializationResult, StructuredDeserializer, StructuredVisitor,
};
use std::borrow::Borrow;
use std::io::Read;

impl StructuredDeserializer for Event {
    fn deserialize_structured<V: StructuredVisitor>(
        self,
        visitor: &mut V,
    ) -> DeserializationResult {
        let vec: Vec<u8> = serde_json::to_vec(&self)?;
        visitor.set_structured_event::<&[u8]>(vec.borrow())
    }
}

impl BinaryDeserializer for Event {
    fn deserialize_binary<V: BinaryVisitor>(self, visitor: &mut V) -> DeserializationResult {
        visitor.set_spec_version(self.get_specversion())?;
        self.attributes.deserialize_attributes(visitor)?;
        for (k, v) in self.extensions.into_iter() {
            visitor.set_extension(&k, v.into())?;
        }
        match self.data.as_ref() {
            Some(Data::String(s)) => visitor.set_body(s.as_bytes()),
            Some(Data::Binary(v)) => visitor.set_body::<&[u8]>(v),
            Some(Data::Json(j)) => {
                let vec: Vec<u8> = serde_json::to_vec(j)?;
                visitor.set_body::<&[u8]>(vec.borrow())
            }
            None => Ok(()),
        }
    }
}

pub(crate) trait AttributesDeserializer {
    fn deserialize_attributes<V: BinaryVisitor>(self, visitor: &mut V) -> DeserializationResult;
}

pub(crate) trait AttributesSerializer {
    fn serialize_attribute(
        &mut self,
        name: &str,
        value: MessageAttributeValue,
    ) -> SerializationResult;
}

impl AttributesDeserializer for Attributes {
    fn deserialize_attributes<V: BinaryVisitor>(self, visitor: &mut V) -> DeserializationResult {
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

impl StructuredVisitor for Event {
    fn set_structured_event<R: Read>(&mut self, reader: R) -> SerializationResult {
        let new_event: Event = serde_json::from_reader(reader)?;
        self.attributes = new_event.attributes;
        self.data = new_event.data;
        self.extensions = new_event.extensions;
        Ok(())
    }
}

impl BinaryVisitor for Event {
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

    fn set_body<R: Read>(&mut self, mut reader: R) -> SerializationResult {
        let mut v = Vec::new();
        let _ = reader.read_to_end(&mut v)?;
        self.data = Some(Data::from_binary(self.get_datacontenttype(), v)?);
        Ok(())
    }
}
