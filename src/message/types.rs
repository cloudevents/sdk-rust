use chrono::{DateTime, Utc};

pub enum MessageAttributeValue {
    Boolean(bool),
    Integer(i64),
    String(String),
    Binary(Vec<u8>),
    Uri(String),
    UriRef(String),
    DateTime(DateTime<Utc>),
}