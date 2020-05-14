use crate::event::SpecVersion;
use crate::Event;
use std::io::Read;
use super::{Encoding, MessageAttributeValue, DeserializationResult, SerializationResult, Error, BinarySerializer, StructuredSerializer};

pub trait StructuredDeserializer
where
    Self: Sized,
{
    fn deserialize_structured<R: Sized, V: StructuredSerializer<R>>(self, serializer: V) -> Result<R, Error>;

    fn into_event(self) -> Result<Event, Error> {
        self.deserialize_structured(Event::default())
    }
}

pub trait BinaryDeserializer
where
    Self: Sized,
{
    fn deserialize_binary<R: Sized, V: BinarySerializer<R>>(self, serializer: V) -> Result<R, Error>;

    fn into_event(self) -> Result<Event, Error> {
        self.deserialize_binary(Event::default())
    }
}

pub trait MessageDeserializer
where
    Self: StructuredDeserializer + BinaryDeserializer + Sized,
{
    fn encoding(&self) -> Encoding;

    fn into_event(self) -> Result<Event, Error> {
        let mut ev = Event::default();
        self.deserialize_to(ev)
    }

    fn deserialize_to_binary<R: Sized, T: BinarySerializer<R>>(self, mut serializer: T) -> Result<R, Error> {
        if self.encoding() == Encoding::BINARY {
            return self.deserialize_binary(serializer);
        }

        let ev = MessageDeserializer::into_event(self)?;
        return ev.deserialize_binary(serializer);
    }

    fn deserialize_to_structured<R: Sized, T: StructuredSerializer<R>>(
        self,
        mut serializer: T,
    ) -> Result<R, Error> {
        if self.encoding() == Encoding::STRUCTURED {
            return self.deserialize_structured(serializer);
        }

        let ev = MessageDeserializer::into_event(self)?;
        return ev.deserialize_structured(serializer);
    }

    fn deserialize_to<R: Sized, T: BinarySerializer<R> + StructuredSerializer<R>>(
        self,
        mut serializer: T,
    ) -> Result<R, Error> {
        if self.encoding() == Encoding::STRUCTURED {
            self.deserialize_structured(serializer)
        } else {
            self.deserialize_binary(serializer)
        }
    }
}
