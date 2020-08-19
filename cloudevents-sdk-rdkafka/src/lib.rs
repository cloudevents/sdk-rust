//! This library provides Kafka protocol bindings for CloudEvents
//! using the [rust-rdkafka](https://fede1024.github.io/rust-rdkafka) library.
//!
//! To produce Cloudevents:
//!
//! ```
//!
//! use cloudevents::Event;
//! use rdkafka::producer::{FutureProducer, FutureRecord};
//! use rdkafka::util::Timeout;
//! use cloudevents_sdk_rdkafka::{MessageRecord, FutureRecordExt};
//!
//! # async fn produce(producer: &FutureProducer, event: Event) {
//! let message_record = MessageRecord::from_event(event)
//!   .expect("error while serializing the event");
//!
//! producer.send(
//!     FutureRecord::to("topic")
//!         .key("some_event")
//!         .message_record(&message_record),
//!     Timeout::never
//! ).await;
//!
//! # }
//!
//! ```
//!
//! To consume Cloudevents:
//!
//! ```
//! use rdkafka::consumer::{StreamConsumer, DefaultConsumerContext, Consumer, CommitMode};
//! use cloudevents_sdk_rdkafka::MessageExt;
//! use futures::StreamExt;
//!
//! # async fn consume(consumer: StreamConsumer<DefaultConsumerContext>) {
//! let mut message_stream = consumer.start();
//!
//! while let Some(message) = message_stream.next().await {
//!     match message {
//!         Err(e) => println!("Kafka error: {}", e),
//!         Ok(m) => {
//!             let event = m.to_event().expect("error while deserializing record to CloudEvent");
//!             println!("Received Event: {:#?}", event);
//!             consumer.commit_message(&m, CommitMode::Async).unwrap();
//!         }
//!     };
//! }
//! # }
//! ```

#[macro_use]
mod headers;
mod kafka_consumer_record;
mod kafka_producer_record;

pub use kafka_consumer_record::record_to_event;
pub use kafka_consumer_record::ConsumerRecordDeserializer;
pub use kafka_consumer_record::MessageExt;

pub use kafka_producer_record::BaseRecordExt;
pub use kafka_producer_record::FutureRecordExt;
pub use kafka_producer_record::MessageRecord;
