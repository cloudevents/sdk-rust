#[macro_use]
mod headers;
mod kafka_consumer_record;
mod kafka_producer_record;

pub use kafka_consumer_record::BorrowedMessageExt;
pub use kafka_consumer_record::OwnedMessageExt;
pub use kafka_producer_record::EventExt;
pub use kafka_producer_record::FutureRecordExt;
