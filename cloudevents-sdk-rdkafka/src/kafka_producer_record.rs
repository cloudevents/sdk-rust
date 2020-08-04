use super::headers;
use cloudevents::event::SpecVersion;
use cloudevents::message::{
    BinaryDeserializer, BinarySerializer, MessageAttributeValue, Result, StructuredSerializer,
};
use cloudevents::Event;
use rdkafka::message::{OwnedHeaders, ToBytes};
use rdkafka::producer::FutureRecord;

/// Wrapper for [`RequestBuilder`] that implements [`StructuredSerializer`] & [`BinarySerializer`] traits
#[derive(Debug)]
pub struct ProducerRecordSerializer {
    pub(crate) payload: Option<Vec<u8>>,
    pub(crate) headers: OwnedHeaders,
}

impl ProducerRecordSerializer {
    pub fn new() -> ProducerRecordSerializer {
        ProducerRecordSerializer {
            payload: None,
            headers: OwnedHeaders::new(),
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
                })
                .unwrap()
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

/// Method to fill a [`RequestBuilder`] with an [`Event`]
pub fn event_to_record<'a, K: ToBytes + ?Sized>(
    event: &'a ProducerRecordSerializer,
    mut record: FutureRecord<'a, K, Vec<u8>>,
) -> Result<FutureRecord<'a, K, Vec<u8>>> {
    let header = event.headers.clone();

    record = record.headers(header);

    match event.payload.as_ref() {
        Some(s) => record = record.payload(s),
        None => record = record,
    }

    Ok(record)
}

/// Extension Trait for [`RequestBuilder`] which acts as a wrapper for the function [`event_to_request()`]
pub trait FutureRecordExt<'a, K: ToBytes + ?Sized> {
    fn event(self, event: &'a ProducerRecordSerializer) -> Result<FutureRecord<'a, K, Vec<u8>>>;
}

impl<'a, K: ToBytes + ?Sized> FutureRecordExt<'a, K> for FutureRecord<'a, K, Vec<u8>> {
    fn event(self, event: &'a ProducerRecordSerializer) -> Result<FutureRecord<'a, K, Vec<u8>>> {
        event_to_record(event, self)
    }
}

pub trait EventExt {
    fn serialize_event(self) -> Result<ProducerRecordSerializer>;
}

impl EventExt for Event {
    fn serialize_event(self) -> Result<ProducerRecordSerializer> {
        BinaryDeserializer::deserialize_binary(self, ProducerRecordSerializer::new())
    }
}
