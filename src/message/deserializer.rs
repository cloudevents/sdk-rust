use crate::event::SpecVersion;
use crate::message::types::MessageAttributeValue;

pub trait StructuredDeserializer {
    fn deserialize_structured<V: StructuredVisitor>(&self, visitor: &mut V);
}

pub trait StructuredVisitor {
    fn visit_structured_event(&mut self, )
}

pub trait BinaryDeserializer {
    fn deserialize_binary<V: BinaryVisitor>(&self, visitor: &mut V);
}

pub trait BinaryVisitor {
    fn start(&mut self);

    fn set_spec_version(&mut self, spec_version: SpecVersion);

    fn set_attribute(&mut self, attribute_name: str, attribute_value: MessageAttributeValue);

    fn set_data(&mut self);

    fn end(&mut self);
}