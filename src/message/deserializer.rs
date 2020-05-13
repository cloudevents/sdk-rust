use super::{Encoding, MessageAttributeValue};
use crate::event::SpecVersion;
use crate::Event;
use snafu::Snafu;
use std::io::Read;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Wrong encoding"))]
    WrongEncoding { },
    #[snafu(display("{}", source))]
    #[snafu(context(false))]
    InvalidSpecVersion { source: crate::event::spec_version::InvalidSpecVersion },
    #[snafu(display("Unrecognized attribute name: {}", name))]
    UnrecognizedAttributeName { name: String },
    #[snafu(display("Error while parsing a time string: {}", source))]
    #[snafu(context(false))]
    ParseTimeError { source: chrono::ParseError },
    #[snafu(display("Error while parsing a url: {}", source))]
    #[snafu(context(false))]
    ParseUrlError { source: url::ParseError },
    #[snafu(display("Error while decoding base64: {}", source))]
    #[snafu(context(false))]
    Base64DecodingError { source: base64::DecodeError },
    #[snafu(display("Error while serializing/deserializing to json: {}", source))]
    #[snafu(context(false))]
    SerdeJsonError { source: serde_json::Error },
    #[snafu(display("IO Error: {}", source))]
    #[snafu(context(false))]
    IOError { source: std::io::Error },
    #[snafu(display("Other error: {}", source))]
    Other { source: Box<dyn std::error::Error> },
}

pub type SerializationResult = Result<(), Error>;
pub type DeserializationResult = Result<(), Error>;

pub trait StructuredDeserializer
where
    Self: Sized,
{
    fn deserialize_structured<V: StructuredSerializer>(self, serializer: &mut V) -> DeserializationResult;

    fn into_event(self) -> Result<Event, Error> {
        let mut ev = Event::default();
        self.deserialize_structured(&mut ev)?;
        Ok(ev)
    }
}

pub trait StructuredSerializer {
    fn set_structured_event<R: Read>(&mut self, reader: R) -> SerializationResult;
}

pub trait BinaryDeserializer
where
    Self: Sized,
{
    fn deserialize_binary<V: BinarySerializer>(self, serializer: &mut V) -> DeserializationResult;

    fn into_event(self) -> Result<Event, Error> {
        let mut ev = Event::default();
        self.deserialize_binary(&mut ev)?;
        Ok(ev)
    }
}

pub trait BinarySerializer {
    fn set_spec_version(&mut self, spec_version: SpecVersion) -> SerializationResult;

    fn set_attribute(&mut self, name: &str, value: MessageAttributeValue) -> SerializationResult;

    fn set_extension(&mut self, name: &str, value: MessageAttributeValue) -> SerializationResult;

    fn set_body<R: Read>(&mut self, reader: R) -> SerializationResult;
}

pub trait MessageDeserializer
where
    Self: StructuredDeserializer + BinaryDeserializer + Sized,
{
    fn encoding(&self) -> Encoding;

    fn into_event(self) -> Result<Event, Error> {
        let mut ev = Event::default();
        self.deserialize_to(&mut ev)?;
        Ok(ev)
    }

    fn deserialize_to_binary<T: BinarySerializer>(self, serializer: &mut T) -> DeserializationResult {
        if self.encoding() == Encoding::BINARY {
            return self.deserialize_binary(serializer);
        }

        let ev = MessageDeserializer::into_event(self)?;
        return ev.deserialize_binary(serializer);
    }

    fn deserialize_to_structured<T: StructuredSerializer>(
        self,
        serializer: &mut T,
    ) -> DeserializationResult {
        if self.encoding() == Encoding::STRUCTURED {
            return self.deserialize_structured(serializer);
        }

        let ev = MessageDeserializer::into_event(self)?;
        return ev.deserialize_structured(serializer);
    }

    fn deserialize_to<T: BinarySerializer + StructuredSerializer>(
        self,
        serializer: &mut T,
    ) -> DeserializationResult {
        if self.encoding() == Encoding::STRUCTURED {
            self.deserialize_structured(serializer)
        } else {
            self.deserialize_binary(serializer)
        }
    }
}
