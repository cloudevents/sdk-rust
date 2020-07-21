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
    payload: Option<Vec<u8>>,
    headers: OwnedHeaders,
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
        self.headers = self.headers.add("ce_specversion", spec_version.as_str());

        Ok(self)
    }

    fn set_attribute(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.headers = self.headers.add(
            &headers::ATTRIBUTES_TO_HEADERS.get(name).unwrap().clone()[..],
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
            .add("content-type", "application/cloudevents+json");

        self.payload = Some(bytes);

        Ok(self)
    }
}

/// Method to fill a [`RequestBuilder`] with an [`Event`]
pub fn event_to_record<'a, K: ToBytes + ?Sized>(
    event: &'a ProducerRecordSerializer,
    mut record: FutureRecord<'a, K, Vec<u8>>,
) -> Result<FutureRecord<'a, K, Vec<u8>>> {
    //let serialized_request = BinaryDeserializer::deserialize_binary(event, ProducerRecordSerializer::new())?;
    let header = event.headers.clone();

    record = record.headers(header);
    record = record.payload(event.payload.as_ref().unwrap());

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

/*
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    use clap::{App, Arg};
    use log::info;

    use rdkafka::config::ClientConfig;
    use rdkafka::message::OwnedHeaders;
    use rdkafka::producer::{FutureProducer, FutureRecord};
    use rdkafka::util::get_rdkafka_version;

    use crate::example_utils::setup_logger;

    mod example_utils;

    async fn produce(brokers: &str, topic_name: &str) {
        let producer: &FutureProducer = &ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Producer creation error");

        // This loop is non blocking: all messages will be sent one after the other, without waiting
        // for the results.
        let futures = (0..5)
            .map(|i| async move {
                // The send operation on the topic returns a future, which will be
                // completed once the result or failure from Kafka is received.
                let delivery_status = producer
                    .send(
                        FutureRecord::to(topic_name)
                            .payload(&format!("Message {}", i))
                            .key(&format!("Key {}", i))
                            .headers(OwnedHeaders::new().add("header_key", "header_value")),
                        Duration::from_secs(0),
                    )
                    .await;

                // This will be executed when the result is received.
                info!("Delivery status for message {} received", i);
                delivery_status
            })
            .collect::<Vec<_>>();

        // This loop will wait until all delivery statuses have been received.
        for future in futures {
            info!("Future completed. Result: {:?}", future.await);
        }
    }

    #[tokio::test]
    async fn test_record() {
        let matches = App::new("producer example")
            .version(option_env!("CARGO_PKG_VERSION").unwrap_or(""))
            .about("Simple command line producer")
            .arg(
                Arg::with_name("brokers")
                    .short("b")
                    .long("brokers")
                    .help("Broker list in kafka format")
                    .takes_value(true)
                    .default_value("localhost:9092"),
            )
            .arg(
                Arg::with_name("log-conf")
                    .long("log-conf")
                    .help("Configure the logging format (example: 'rdkafka=trace')")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("topic")
                    .short("t")
                    .long("topic")
                    .help("Destination topic")
                    .takes_value(true)
                    .required(true),
            )
            .get_matches();

        setup_logger(true, matches.value_of("log-conf"));

        let (version_n, version_s) = get_rdkafka_version();
        info!("rd_kafka_version: 0x{:08x}, {}", version_n, version_s);

        let topic = matches.value_of("topic").unwrap();
        let brokers = matches.value_of("brokers").unwrap();

        produce(brokers, topic).await;
    }

    #[tokio::test]
    async fn test_request() {
        let url = mockito::server_url();
        let m = mock("POST", "/")
            .match_header("ce-specversion", "1.0")
            .match_header("ce-id", "0001")
            .match_header("ce-type", "example.test")
            .match_header("ce-source", "http://localhost/")
            .match_header("ce-someint", "10")
            .match_body(Matcher::Missing)
            .create();

        let input = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .source(Url::from_str("http://localhost/").unwrap())
            .extension("someint", "10")
            .build()
            .unwrap();

        let client = reqwest::Client::new();
        client
            .post(&url)
            .event(input)
            .unwrap()
            .send()
            .await
            .unwrap();

        m.assert();
    }

    #[tokio::test]
    async fn test_request_with_full_data() {
        let j = json!({"hello": "world"});

        let url = mockito::server_url();
        let m = mock("POST", "/")
            .match_header("ce-specversion", "1.0")
            .match_header("ce-id", "0001")
            .match_header("ce-type", "example.test")
            .match_header("ce-source", "http://localhost/")
            .match_header("content-type", "application/json")
            .match_header("ce-someint", "10")
            .match_body(Matcher::Exact(j.to_string()))
            .create();

        let input = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .source(Url::from_str("http://localhost").unwrap())
            .data("application/json", j.clone())
            .extension("someint", "10")
            .build()
            .unwrap();

        let client = reqwest::Client::new();

        client
            .post(&url)
            .event(input)
            .unwrap()
            .send()
            .await
            .unwrap();

        m.assert();
    }

    #[tokio::test]
    async fn test_structured_request_with_full_data() {
        let j = json!({"hello": "world"});

        let input = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .source(Url::from_str("http://localhost").unwrap())
            .data("application/json", j.clone())
            .extension("someint", "10")
            .build()
            .unwrap();

        let url = mockito::server_url();
        let m = mock("POST", "/")
            .match_header("content-type", "application/cloudevents+json")
            .match_body(Matcher::Exact(serde_json::to_string(&input).unwrap()))
            .create();

        let client = reqwest::Client::new();
        StructuredDeserializer::deserialize_structured(
            input,
            ProducerRecordSerializer::new(client.post(&url)),
        )
        .unwrap()
        .send()
        .await
        .unwrap();

        m.assert();
    }
}*/
