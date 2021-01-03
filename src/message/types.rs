use crate::event::ExtensionValue;
use chrono::{DateTime, Utc};
use std::convert::TryInto;
use std::fmt;
use std::prelude::v1::*;

#[cfg(feature = "std")]
use url::Url;
#[cfg(not(feature = "std"))]
use String as Url;

/// Union type representing a [CloudEvent context attribute type](https://github.com/cloudevents/spec/blob/v1.0/spec.md#type-system).
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum MessageAttributeValue {
    Boolean(bool),
    Integer(i64),
    String(String),
    Binary(Vec<u8>),
    Uri(Url),
    UriRef(Url),
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

impl TryInto<Url> for MessageAttributeValue {
    type Error = super::Error;

    #[cfg(feature = "std")]
    fn try_into(self) -> Result<Url, Self::Error> {
        match self {
            MessageAttributeValue::Uri(u) => Ok(u),
            MessageAttributeValue::UriRef(u) => Ok(u),
            v => Ok(Url::parse(v.to_string().as_ref())?),
        }
    }
    #[cfg(not(feature = "std"))]
    fn try_into(self) -> Result<Url, Self::Error> {
        match self {
            MessageAttributeValue::Uri(u) => Ok(u),
            MessageAttributeValue::UriRef(u) => Ok(u),
            v => Ok(v.to_string()),
        }
    }
}

impl fmt::Display for MessageAttributeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageAttributeValue::Boolean(b) => write!(f, "{}", b),
            MessageAttributeValue::Integer(i) => write!(f, "{}", i),
            MessageAttributeValue::String(s) => write!(f, "{}", s),
            MessageAttributeValue::Binary(v) => write!(f, "{}", base64::encode(v)),
            MessageAttributeValue::Uri(u) => write!(f, "{}", u.to_string()),
            MessageAttributeValue::UriRef(u) => write!(f, "{}", u.to_string()),
            MessageAttributeValue::DateTime(d) => write!(f, "{}", d.to_rfc3339()),
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
