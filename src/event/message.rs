use super::Data;
use super::Event;
use super::{Attributes, AttributesReader};
use crate::event::SpecVersion;
use crate::message::{
    BinaryDeserializer, BinarySerializer, Result, MessageAttributeValue,
    StructuredDeserializer, StructuredSerializer,
};

impl StructuredDeserializer for Event {
    fn deserialize_structured<R, V: StructuredSerializer<R>>(self, visitor: V) -> Result<R> {
        let vec: Vec<u8> = serde_json::to_vec(&self)?;
        visitor.set_structured_event(vec)
    }
}

impl BinaryDeserializer for Event {
    fn deserialize_binary<R: Sized, V: BinarySerializer<R>>(
        self,
        mut visitor: V,
    ) -> Result<R> {
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
    fn deserialize_attributes<R: Sized, V: BinarySerializer<R>>(
        self,
        visitor: V,
    ) -> Result<V>;
}

pub(crate) trait AttributesSerializer {
    fn serialize_attribute(
        &mut self,
        name: &str,
        value: MessageAttributeValue,
    ) -> Result<()>;
}

impl AttributesDeserializer for Attributes {
    fn deserialize_attributes<R: Sized, V: BinarySerializer<R>>(
        self,
        visitor: V,
    ) -> Result<V> {
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
    ) -> Result<()> {
        match self {
            Attributes::V03(v03) => v03.serialize_attribute(name, value),
            Attributes::V10(v10) => v10.serialize_attribute(name, value),
        }
    }
}

impl StructuredSerializer<Event> for Event {
    fn set_structured_event(mut self, bytes: Vec<u8>) -> Result<Event> {
        let new_event: Event = serde_json::from_slice(&bytes)?;
        self.attributes = new_event.attributes;
        self.data = new_event.data;
        self.extensions = new_event.extensions;
        Ok(self)
    }
}

impl BinarySerializer<Event> for Event {
    fn set_spec_version(mut self, spec_version: SpecVersion) -> Result<Self> {
        match spec_version {
            SpecVersion::V03 => self.attributes = self.attributes.clone().into_v03(),
            SpecVersion::V10 => self.attributes = self.attributes.clone().into_v10(),
        }
        Ok(self)
    }

    fn set_attribute(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.attributes.serialize_attribute(name, value)?;
        Ok(self)
    }

    fn set_extension(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.extensions.insert(name.to_string(), value.into());
        Ok(self)
    }

    fn end_with_data(mut self, bytes: Vec<u8>) -> Result<Event> {
        self.data = Some(Data::from_binary(self.get_datacontenttype(), bytes)?);
        Ok(self)
    }

    fn end(self) -> Result<Event> {
        Ok(self)
    }
}
