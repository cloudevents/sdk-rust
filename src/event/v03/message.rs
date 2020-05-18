use crate::message::{
    BinarySerializer, DeserializationResult, Error, MessageAttributeValue, SerializationResult,
};
use std::convert::TryInto;

impl crate::event::message::AttributesDeserializer for super::Attributes {
    fn deserialize_attributes<R: Sized, V: BinarySerializer<R>>(
        self,
        visitor: &mut V,
    ) -> DeserializationResult {
        visitor.set_attribute("id", MessageAttributeValue::String(self.id))?;
        visitor.set_attribute("type", MessageAttributeValue::String(self.ty))?;
        visitor.set_attribute("source", MessageAttributeValue::UriRef(self.source))?;
        if self.datacontenttype.is_some() {
            visitor.set_attribute(
                "datacontenttype",
                MessageAttributeValue::String(self.datacontenttype.unwrap()),
            )?;
        }
        if self.schemaurl.is_some() {
            visitor.set_attribute(
                "schemaurl",
                MessageAttributeValue::Uri(self.schemaurl.unwrap()),
            )?;
        }
        if self.subject.is_some() {
            visitor.set_attribute(
                "subject",
                MessageAttributeValue::String(self.subject.unwrap()),
            )?;
        }
        if self.time.is_some() {
            visitor.set_attribute("time", MessageAttributeValue::DateTime(self.time.unwrap()))?;
        }
        Ok(())
    }
}

impl crate::event::message::AttributesSerializer for super::Attributes {
    fn serialize_attribute(
        &mut self,
        name: &str,
        value: MessageAttributeValue,
    ) -> SerializationResult {
        match name {
            "id" => self.id = value.to_string(),
            "type" => self.ty = value.to_string(),
            "source" => self.source = value.try_into()?,
            "datacontenttype" => self.datacontenttype = Some(value.to_string()),
            "schemaurl" => self.schemaurl = Some(value.try_into()?),
            "subject" => self.subject = Some(value.to_string()),
            "time" => self.time = Some(value.try_into()?),
            _ => {
                return Err(Error::UnrecognizedAttributeName {
                    name: name.to_string(),
                })
            }
        };
        Ok(())
    }
}
