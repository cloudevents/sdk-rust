use super::{SerializationResult, MessageAttributeValue, Error};
use crate::event::SpecVersion;

pub trait StructuredSerializer<RETURN: Sized> {
    fn set_structured_event(self, bytes: Vec<u8>) -> Result<RETURN, Error>;
}

pub trait BinarySerializer<RETURN: Sized> {
    fn set_spec_version(&mut self, spec_version: SpecVersion) -> SerializationResult;

    fn set_attribute(&mut self, name: &str, value: MessageAttributeValue) -> SerializationResult;

    fn set_extension(&mut self, name: &str, value: MessageAttributeValue) -> SerializationResult;

    fn end_with_data(self, bytes: Vec<u8>) -> Result<RETURN, Error>;

    fn end(self) -> Result<RETURN, Error>;
}