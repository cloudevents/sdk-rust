use super::headers;
use cloudevents::event::SpecVersion;
use cloudevents::message::{
    BinaryDeserializer, BinarySerializer, MessageAttributeValue, Result, StructuredSerializer,
};
use cloudevents::Event;
use rdkafka::message::{OwnedHeaders, ToBytes};
use rdkafka::producer::{BaseRecord, FutureRecord};

/// This struct contains a serialized CloudEvent message in the Kafka shape.
/// Implements [`StructuredSerializer`] & [`BinarySerializer`] traits.
///
/// To instantiate a new `MessageRecord` from an [`Event`],
/// look at [`Self::from_event`] or use [`StructuredDeserializer::deserialize_structured`](cloudevents::message::StructuredDeserializer::deserialize_structured)
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
    fn set_spec_version(mut self, spec_version: SpecVersion) -> Result<Self> {
        self.headers = self
            .headers
            .add(headers::SPEC_VERSION_HEADER, spec_version.as_str());

        Ok(self)
    }

    fn set_attribute(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.headers = self.headers.add(
            &headers::ATTRIBUTES_TO_HEADERS
                .get(name)
                .ok_or(cloudevents::message::Error::UnknownAttribute {
                    name: String::from(name),
                })?
                .clone()[..],
            &value.to_string()[..],
        );

        Ok(self)
    }

    fn set_extension(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.headers = self
            .headers
            .add(&attribute_name_to_header!(name)[..], &value.to_string()[..]);

        Ok(self)
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
        self.headers = self
            .headers
            .add(headers::CONTENT_TYPE, headers::CLOUDEVENTS_JSON_HEADER);

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
