use super::{MessageAttributeValue, Result};
use crate::event::SpecVersion;

pub trait StructuredSerializer<RETURN: Sized> {
    fn set_structured_event(self, bytes: Vec<u8>) -> Result<RETURN>;
}

pub trait BinarySerializer<RETURN: Sized>
where
    Self: Sized,
{
    fn set_spec_version(self, spec_version: SpecVersion) -> Result<Self>;

    fn set_attribute(self, name: &str, value: MessageAttributeValue) -> Result<Self>;

    fn set_extension(self, name: &str, value: MessageAttributeValue) -> Result<Self>;

    fn end_with_data(self, bytes: Vec<u8>) -> Result<RETURN>;

    fn end(self) -> Result<RETURN>;
}
