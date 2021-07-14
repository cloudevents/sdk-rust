//! This library provides Kafka protocol bindings for CloudEvents
//! using the [rust-rdkafka](https://fede1024.github.io/rust-rdkafka) library.
//!
//! To produce Cloudevents:
//!
//! ```
//! # use rdkafka_lib as rdkafka;
//! use cloudevents::Event;
//! use rdkafka::producer::{FutureProducer, FutureRecord};
//! use rdkafka::util::Timeout;
//! use cloudevents::binding::rdkafka::{MessageRecord, FutureRecordExt};
//!
//! # async fn produce(producer: &FutureProducer, event: Event) -> Result<(), Box<dyn std::error::Error>> {
//! let message_record = MessageRecord::from_event(event)?;
//!
//! producer.send(
//!     FutureRecord::to("topic")
//!         .key("some_event")
//!         .message_record(&message_record),
//!     Timeout::Never
//! ).await;
//! # Ok(())
//! # }
//!
//! ```
//!
//! To consume Cloudevents:
//!
//! ```
//! # use rdkafka_lib as rdkafka;
//! use rdkafka::consumer::{StreamConsumer, DefaultConsumerContext, Consumer, CommitMode};
//! use cloudevents::binding::rdkafka::MessageExt;
//! use futures::StreamExt;
//!
//! # async fn consume(consumer: StreamConsumer<DefaultConsumerContext>) -> Result<(), Box<dyn std::error::Error>> {
//! let mut message_stream = consumer.start();
//!
//! while let Some(message) = message_stream.next().await {
//!     match message {
//!         Err(e) => println!("Kafka error: {}", e),
//!         Ok(m) => {
//!             let event = m.to_event()?;
//!             println!("Received Event: {}", event);
//!             consumer.commit_message(&m, CommitMode::Async)?;
//!         }
//!     };
//! }
//! # Ok(())
//! # }
//! ```

#![deny(broken_intra_doc_links)]

mod kafka_consumer_record;
mod kafka_producer_record;

pub use kafka_consumer_record::record_to_event;
pub use kafka_consumer_record::ConsumerRecordDeserializer;
pub use kafka_consumer_record::MessageExt;

pub use kafka_producer_record::BaseRecordExt;
pub use kafka_producer_record::FutureRecordExt;
pub use kafka_producer_record::MessageRecord;
