//! This library provides Kafka protocol bindings for [`CloudEvents`](https://cloudevents.io/) 
//! using the [`rust-rdkafka`](https://docs.rs/rdkafka) library. It is a part of [`cloudevents-sdk`](https://docs.rs/cloudevents-sdk).
 
#[macro_use]
mod headers;
mod kafka_consumer_record;
mod kafka_producer_record;

pub use kafka_consumer_record::BorrowedMessageExt;
pub use kafka_consumer_record::OwnedMessageExt;
pub use kafka_producer_record::EventExt;
pub use kafka_producer_record::FutureRecordExt;
