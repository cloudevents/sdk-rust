use super::{StructuredDeserializer, BinaryDeserializer, Encoding};

pub trait MessageDeserializer where Self: StructuredDeserializer + BinaryDeserializer {
    fn encoding(&self) -> Encoding;
}