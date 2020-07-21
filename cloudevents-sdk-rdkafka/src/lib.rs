#[macro_use]
mod headers;
mod kafka_consumer_record;
mod kafka_producer_record;

pub use kafka_producer_record::EventExt as EventExt;
pub use kafka_producer_record::FutureRecordExt as FutureRecordExt;
pub use kafka_consumer_record::OwnedMessageExt as OwnedMessageExt;