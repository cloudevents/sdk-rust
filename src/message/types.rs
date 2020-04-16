use crate::event::ExtensionValue;
use chrono::{DateTime, Utc};
use std::convert::TryInto;

pub enum MessageAttributeValue {
    Boolean(bool),
    Integer(i64),
    String(String),
    Binary(Vec<u8>),
    Uri(String),
    UriRef(String),
    DateTime(DateTime<Utc>),
}

impl TryInto<DateTime<Utc>> for MessageAttributeValue {
    type Error = super::Error;

    fn try_into(self) -> Result<DateTime<Utc>, Self::Error> {
        match self {
            MessageAttributeValue::DateTime(d) => Ok(d),
            v => Ok(DateTime::<Utc>::from(DateTime::parse_from_rfc3339(
                v.to_string().as_ref(),
            )?)),
        }
    }
}

impl ToString for MessageAttributeValue {
    fn to_string(&self) -> String {
        match self {
            MessageAttributeValue::Boolean(b) => b.to_string(),
            MessageAttributeValue::Integer(i) => i.to_string(),
            MessageAttributeValue::String(s) => s.clone(),
            MessageAttributeValue::Binary(v) => base64::encode(v),
            MessageAttributeValue::Uri(s) => s.clone(),
            MessageAttributeValue::UriRef(s) => s.clone(),
            MessageAttributeValue::DateTime(d) => d.to_rfc3339(),
        }
    }
}

impl Into<MessageAttributeValue> for ExtensionValue {
    fn into(self) -> MessageAttributeValue {
        match self {
            ExtensionValue::String(s) => MessageAttributeValue::String(s),
            ExtensionValue::Boolean(b) => MessageAttributeValue::Boolean(b),
            ExtensionValue::Integer(i) => MessageAttributeValue::Integer(i),
        }
    }
}

impl Into<ExtensionValue> for MessageAttributeValue {
    fn into(self) -> ExtensionValue {
        match self {
            MessageAttributeValue::Integer(i) => ExtensionValue::Integer(i),
            MessageAttributeValue::Boolean(b) => ExtensionValue::Boolean(b),
            v => ExtensionValue::String(v.to_string()),
        }
    }
}
