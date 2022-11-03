use rdkafka_lib as rdkafka;

use crate::binding::{
    kafka::{header_prefix, SPEC_VERSION_HEADER},
    CLOUDEVENTS_JSON_HEADER, CONTENT_TYPE,
};
use crate::event::SpecVersion;
use crate::message::{
    BinaryDeserializer, BinarySerializer, MessageAttributeValue, Result, StructuredSerializer,
};
use crate::Event;
use rdkafka::message::{Header, OwnedHeaders, ToBytes};
use rdkafka::producer::{BaseRecord, FutureRecord};

/// This struct contains a serialized CloudEvent message in the Kafka shape.
/// Implements [`StructuredSerializer`] & [`BinarySerializer`] traits.
///
/// To instantiate a new `MessageRecord` from an [`Event`],
/// look at [`Self::from_event`] or use [`StructuredDeserializer::deserialize_structured`](crate::message::StructuredDeserializer::deserialize_structured)
/// or [`BinaryDeserializer::deserialize_binary`].
pub struct MessageRecord {
    pub(crate) headers: OwnedHeaders,
    pub(crate) payload: Option<Vec<u8>>,
}

impl MessageRecord {
    /// Create a new empty [`MessageRecord`]
    pub fn new() -> Self {
        MessageRecord {
            headers: OwnedHeaders::new(),
            payload: None,
        }
    }

    /// Create a new [`MessageRecord`], filled with `event` serialized in binary mode.
    pub fn from_event(event: Event) -> Result<Self> {
        BinaryDeserializer::deserialize_binary(event, MessageRecord::new())
    }
}

impl Default for MessageRecord {
    fn default() -> Self {
        Self::new()
    }
}

impl BinarySerializer<MessageRecord> for MessageRecord {
    fn set_spec_version(mut self, sv: SpecVersion) -> Result<Self> {
        let v = sv.to_string();
        let header = Header {
            key: SPEC_VERSION_HEADER,
            value: Some(&v),
        };
        self.headers = self.headers.insert(header);
        Ok(self)
    }

    fn set_attribute(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        let v = value.to_string();
        let header = Header {
            key: &header_prefix(name),
            value: Some(&v),
        };
        self.headers = self.headers.insert(header);
        Ok(self)
    }

    fn set_extension(self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.set_attribute(name, value)
    }

    fn end_with_data(mut self, bytes: Vec<u8>) -> Result<MessageRecord> {
        self.payload = Some(bytes);
        Ok(self)
    }

    fn end(self) -> Result<MessageRecord> {
        Ok(self)
    }
}

impl StructuredSerializer<MessageRecord> for MessageRecord {
    fn set_structured_event(mut self, bytes: Vec<u8>) -> Result<MessageRecord> {
        let header = Header {
            key: CONTENT_TYPE,
            value: Some(CLOUDEVENTS_JSON_HEADER),
        };
        self.headers = self.headers.insert(header);

        self.payload = Some(bytes);

        Ok(self)
    }
}

/// Extension Trait for [`BaseRecord`] that fills the record with a [`MessageRecord`].
///
/// This trait is sealed and cannot be implemented for types outside of this crate.
pub trait BaseRecordExt<'a, K: ToBytes + ?Sized>: private::Sealed {
    /// Fill this [`BaseRecord`] with a [`MessageRecord`].
    fn message_record(
        self,
        message_record: &'a MessageRecord,
    ) -> Result<BaseRecord<'a, K, Vec<u8>>>;
}

impl<'a, K: ToBytes + ?Sized> BaseRecordExt<'a, K> for BaseRecord<'a, K, Vec<u8>> {
    fn message_record(
        mut self,
        message_record: &'a MessageRecord,
    ) -> Result<BaseRecord<'a, K, Vec<u8>>> {
        self = self.headers(message_record.headers.clone());

        if let Some(s) = message_record.payload.as_ref() {
            self = self.payload(s);
        }

        Ok(self)
    }
}

/// Extension Trait for [`FutureRecord`] that fills the record with a [`MessageRecord`].
///
/// This trait is sealed and cannot be implemented for types outside of this crate.
pub trait FutureRecordExt<'a, K: ToBytes + ?Sized>: private::Sealed {
    /// Fill this [`FutureRecord`] with a [`MessageRecord`].
    fn message_record(self, message_record: &'a MessageRecord) -> FutureRecord<'a, K, Vec<u8>>;
}

impl<'a, K: ToBytes + ?Sized> FutureRecordExt<'a, K> for FutureRecord<'a, K, Vec<u8>> {
    fn message_record(mut self, message_record: &'a MessageRecord) -> FutureRecord<'a, K, Vec<u8>> {
        self = self.headers(message_record.headers.clone());

        if let Some(s) = message_record.payload.as_ref() {
            self = self.payload(s);
        }

        self
    }
}

mod private {
    use rdkafka_lib as rdkafka;

    // Sealing the FutureRecordExt and BaseRecordExt
    pub trait Sealed {}
    impl<K: rdkafka::message::ToBytes + ?Sized, V: rdkafka::message::ToBytes> Sealed
        for rdkafka::producer::FutureRecord<'_, K, V>
    {
    }
    impl<K: rdkafka::message::ToBytes + ?Sized, V: rdkafka::message::ToBytes> Sealed
        for rdkafka::producer::BaseRecord<'_, K, V>
    {
    }
}
