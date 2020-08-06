use super::headers;
use cloudevents::event::SpecVersion;
use cloudevents::message::{
    BinaryDeserializer, BinarySerializer, MessageAttributeValue, Result, StructuredSerializer,
};
use cloudevents::Event;
use rdkafka::message::{OwnedHeaders, ToBytes};
use rdkafka::producer::FutureRecord;

/// struct facilitating the creation of a [`FutureRecord`] from an ['Event'].
/// Implements [`StructuredSerializer`] & [`BinarySerializer`] traits.
pub struct ProducerRecordSerializer {
    pub(crate) headers: OwnedHeaders,
    pub(crate) payload: Option<Vec<u8>>,
}

impl ProducerRecordSerializer {
    pub fn new() -> ProducerRecordSerializer {
        ProducerRecordSerializer {
            headers: OwnedHeaders::new(),
            payload: None,
        }
    }
}

impl BinarySerializer<ProducerRecordSerializer> for ProducerRecordSerializer {
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
                .ok_or(cloudevents::message::Error::UnrecognizedAttributeName {
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

    fn end_with_data(mut self, bytes: Vec<u8>) -> Result<ProducerRecordSerializer> {
        self.payload = Some(bytes);

        Ok(self)
    }

    fn end(self) -> Result<ProducerRecordSerializer> {
        Ok(self)
    }
}

impl StructuredSerializer<ProducerRecordSerializer> for ProducerRecordSerializer {
    fn set_structured_event(mut self, bytes: Vec<u8>) -> Result<ProducerRecordSerializer> {
        self.headers = self
            .headers
            .add(headers::CONTENT_TYPE, headers::CLOUDEVENTS_JSON_HEADER);

        self.payload = Some(bytes);

        Ok(self)
    }
}

/// Method to fill a [`FutureRecord`] with an [`Event`]
pub fn event_to_record<'a, K: ToBytes + ?Sized>(
    event: &'a ProducerRecordSerializer,
    mut record: FutureRecord<'a, K, Vec<u8>>,
) -> Result<FutureRecord<'a, K, Vec<u8>>> {
    let header = event.headers.clone();

    record = record.headers(header);

    if let Some(s) = event.payload.as_ref() {
        record = record.payload(s)
    }

    Ok(record)
}

/// Extension Trait for [`FutureRecord`] which acts as a wrapper for the function [`event_to_record()`]:: method.event_to_record.html
pub trait FutureRecordExt<'a, K: ToBytes + ?Sized> {
    /// Generates [`FutureRecord`] from
    /// [`Event`]
    fn event(self, event: &'a ProducerRecordSerializer) -> Result<FutureRecord<'a, K, Vec<u8>>>;
}

impl<'a, K: ToBytes + ?Sized> FutureRecordExt<'a, K> for FutureRecord<'a, K, Vec<u8>> {
    fn event(self, event: &'a ProducerRecordSerializer) -> Result<FutureRecord<'a, K, Vec<u8>>> {
        event_to_record(event, self)
    }
}

/// Extention Trait for [`Event`]
/// for producing a [`ProducerRecordSerializer`] by transforming the provided Event struct
pub trait EventExt {
    /// Generates [`ProducerRecordSerializer`] from [`Event`]
    fn serialize_event(self) -> Result<ProducerRecordSerializer>;
}

impl EventExt for Event {
    fn serialize_event(self) -> Result<ProducerRecordSerializer> {
        BinaryDeserializer::deserialize_binary(self, ProducerRecordSerializer::new())
    }
}
