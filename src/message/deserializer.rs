use super::{BinarySerializer, Encoding, Result, StructuredSerializer};
use crate::Event;

/// Deserializer trait for a Message that can be encoded as structured mode
pub trait StructuredDeserializer
where
    Self: Sized,
{
    /// Deserialize the message to [`StructuredSerializer`]
    fn deserialize_structured<R: Sized, V: StructuredSerializer<R>>(
        self,
        serializer: V,
    ) -> Result<R>;

    /// Convert this Message to [`Event`]
    fn into_event(self) -> Result<Event> {
        self.deserialize_structured(Event::default())
    }
}

/// Deserializer trait for a Message that can be encoded as binary mode
pub trait BinaryDeserializer
where
    Self: Sized,
{
    /// Deserialize the message to [`BinarySerializer`]
    fn deserialize_binary<R: Sized, V: BinarySerializer<R>>(self, serializer: V) -> Result<R>;

    /// Convert this Message to [`Event`]
    fn into_event(self) -> Result<Event> {
        self.deserialize_binary(Event::default())
    }
}

/// Deserializer trait for a Message that can be encoded both in structured mode or binary mode
pub trait MessageDeserializer
where
    Self: StructuredDeserializer + BinaryDeserializer + Sized,
{
    /// Get this message [`Encoding`]
    fn encoding(&self) -> Encoding;

    /// Convert this Message to [`Event`]
    fn into_event(self) -> Result<Event> {
        self.deserialize_to(Event::default())
    }

    /// Deserialize the message to [`BinarySerializer`]
    fn deserialize_to_binary<R: Sized, T: BinarySerializer<R>>(self, serializer: T) -> Result<R> {
        if self.encoding() == Encoding::BINARY {
            return self.deserialize_binary(serializer);
        }

        return MessageDeserializer::into_event(self)?.deserialize_binary(serializer);
    }

    /// Deserialize the message to [`StructuredSerializer`]
    fn deserialize_to_structured<R: Sized, T: StructuredSerializer<R>>(
        self,
        serializer: T,
    ) -> Result<R> {
        if self.encoding() == Encoding::STRUCTURED {
            return self.deserialize_structured(serializer);
        }

        return MessageDeserializer::into_event(self)?.deserialize_structured(serializer);
    }

    /// Deserialize the message to a serializer, depending on the message encoding
    /// You can use this method to transcode this message directly to another serializer, without going through [`Event`]
    fn deserialize_to<R: Sized, T: BinarySerializer<R> + StructuredSerializer<R>>(
        self,
        serializer: T,
    ) -> Result<R> {
        if self.encoding() == Encoding::STRUCTURED {
            self.deserialize_structured(serializer)
        } else {
            self.deserialize_binary(serializer)
        }
    }
}
