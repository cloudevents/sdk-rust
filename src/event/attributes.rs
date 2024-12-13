use super::{
    AttributesIntoIteratorV03, AttributesIntoIteratorV10, AttributesV03, AttributesV10,
    ExtensionValue, SpecVersion, UriReference,
};
use base64::prelude::*;
use chrono::{DateTime, Utc};
use serde::Serializer;
use std::fmt;
use url::Url;

/// Enum representing a borrowed value of a CloudEvent attribute.
/// This represents the types defined in the [CloudEvent spec type system](https://github.com/cloudevents/spec/blob/v1.0/spec.md#type-system)
#[derive(Debug, PartialEq, Eq)]
pub enum AttributeValue<'a> {
    Boolean(&'a bool),
    Integer(&'a i64),
    String(&'a str),
    Binary(&'a [u8]),
    URI(&'a Url),
    URIRef(&'a UriReference),
    Time(&'a DateTime<Utc>),
    SpecVersion(SpecVersion),
}

impl<'a> From<&'a ExtensionValue> for AttributeValue<'a> {
    fn from(ev: &'a ExtensionValue) -> Self {
        match ev {
            ExtensionValue::String(s) => AttributeValue::String(s),
            ExtensionValue::Boolean(b) => AttributeValue::Boolean(b),
            ExtensionValue::Integer(i) => AttributeValue::Integer(i),
        }
    }
}

impl fmt::Display for AttributeValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttributeValue::Boolean(b) => f.serialize_bool(**b),
            AttributeValue::Integer(i) => f.serialize_i64(**i),
            AttributeValue::String(s) => f.write_str(s),
            AttributeValue::Binary(b) => f.write_str(&BASE64_STANDARD.encode(b)),
            AttributeValue::URI(s) => f.write_str(s.as_str()),
            AttributeValue::URIRef(s) => f.write_str(s.as_str()),
            AttributeValue::Time(s) => f.write_str(&s.to_rfc3339()),
            AttributeValue::SpecVersion(s) => s.fmt(f),
        }
    }
}

/// Trait to get [CloudEvents Context attributes](https://github.com/cloudevents/spec/blob/master/spec.md#context-attributes).
pub trait AttributesReader {
    /// Get the [id](https://github.com/cloudevents/spec/blob/master/spec.md#id).
    fn id(&self) -> &str;
    /// Get the [source](https://github.com/cloudevents/spec/blob/master/spec.md#source-1).
    fn source(&self) -> &UriReference;
    /// Get the [specversion](https://github.com/cloudevents/spec/blob/master/spec.md#specversion).
    fn specversion(&self) -> SpecVersion;
    /// Get the [type](https://github.com/cloudevents/spec/blob/master/spec.md#type).
    fn ty(&self) -> &str;
    /// Get the [datacontenttype](https://github.com/cloudevents/spec/blob/master/spec.md#datacontenttype).
    fn datacontenttype(&self) -> Option<&str>;
    /// Get the [dataschema](https://github.com/cloudevents/spec/blob/master/spec.md#dataschema).
    fn dataschema(&self) -> Option<&Url>;
    /// Get the [subject](https://github.com/cloudevents/spec/blob/master/spec.md#subject).
    fn subject(&self) -> Option<&str>;
    /// Get the [time](https://github.com/cloudevents/spec/blob/master/spec.md#time).
    fn time(&self) -> Option<&DateTime<Utc>>;
}

/// Trait to set [CloudEvents Context attributes](https://github.com/cloudevents/spec/blob/master/spec.md#context-attributes).
pub trait AttributesWriter {
    /// Set the [id](https://github.com/cloudevents/spec/blob/master/spec.md#id).
    /// Returns the previous value.
    fn set_id(&mut self, id: impl Into<String>) -> String;
    /// Set the [source](https://github.com/cloudevents/spec/blob/master/spec.md#source-1).
    /// Returns the previous value.
    fn set_source(&mut self, source: impl Into<UriReference>) -> UriReference;
    /// Set the [type](https://github.com/cloudevents/spec/blob/master/spec.md#type).
    /// Returns the previous value.
    fn set_type(&mut self, ty: impl Into<String>) -> String;
    /// Set the [subject](https://github.com/cloudevents/spec/blob/master/spec.md#subject).
    /// Returns the previous value.
    fn set_subject(&mut self, subject: Option<impl Into<String>>) -> Option<String>;
    /// Set the [time](https://github.com/cloudevents/spec/blob/master/spec.md#time).
    /// Returns the previous value.
    fn set_time(&mut self, time: Option<impl Into<DateTime<Utc>>>) -> Option<DateTime<Utc>>;
    /// Set the [datacontenttype](https://github.com/cloudevents/spec/blob/master/spec.md#datacontenttype).
    /// Returns the previous value.
    fn set_datacontenttype(&mut self, datacontenttype: Option<impl Into<String>>)
        -> Option<String>;
    /// Set the [dataschema](https://github.com/cloudevents/spec/blob/master/spec.md#dataschema).
    /// Returns the previous value.
    fn set_dataschema(&mut self, dataschema: Option<impl Into<Url>>) -> Option<Url>;
}

pub(crate) trait AttributesConverter {
    fn into_v03(self) -> AttributesV03;
    fn into_v10(self) -> AttributesV10;
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub(crate) enum AttributesIter<'a> {
    IterV03(AttributesIntoIteratorV03<'a>),
    IterV10(AttributesIntoIteratorV10<'a>),
}

impl<'a> Iterator for AttributesIter<'a> {
    type Item = (&'a str, AttributeValue<'a>);
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            AttributesIter::IterV03(a) => a.next(),
            AttributesIter::IterV10(a) => a.next(),
        }
    }
}

/// Union type representing one of the possible context attributes structs
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Attributes {
    V03(AttributesV03),
    V10(AttributesV10),
}

impl AttributesReader for Attributes {
    fn id(&self) -> &str {
        match self {
            Attributes::V03(a) => a.id(),
            Attributes::V10(a) => a.id(),
        }
    }

    fn source(&self) -> &UriReference {
        match self {
            Attributes::V03(a) => a.source(),
            Attributes::V10(a) => a.source(),
        }
    }

    fn specversion(&self) -> SpecVersion {
        match self {
            Attributes::V03(a) => a.specversion(),
            Attributes::V10(a) => a.specversion(),
        }
    }

    fn ty(&self) -> &str {
        match self {
            Attributes::V03(a) => a.ty(),
            Attributes::V10(a) => a.ty(),
        }
    }

    fn datacontenttype(&self) -> Option<&str> {
        match self {
            Attributes::V03(a) => a.datacontenttype(),
            Attributes::V10(a) => a.datacontenttype(),
        }
    }

    fn dataschema(&self) -> Option<&Url> {
        match self {
            Attributes::V03(a) => a.dataschema(),
            Attributes::V10(a) => a.dataschema(),
        }
    }

    fn subject(&self) -> Option<&str> {
        match self {
            Attributes::V03(a) => a.subject(),
            Attributes::V10(a) => a.subject(),
        }
    }

    fn time(&self) -> Option<&DateTime<Utc>> {
        match self {
            Attributes::V03(a) => a.time(),
            Attributes::V10(a) => a.time(),
        }
    }
}

impl AttributesWriter for Attributes {
    fn set_id(&mut self, id: impl Into<String>) -> String {
        match self {
            Attributes::V03(a) => a.set_id(id),
            Attributes::V10(a) => a.set_id(id),
        }
    }

    fn set_source(&mut self, source: impl Into<UriReference>) -> UriReference {
        match self {
            Attributes::V03(a) => a.set_source(source),
            Attributes::V10(a) => a.set_source(source),
        }
    }

    fn set_type(&mut self, ty: impl Into<String>) -> String {
        match self {
            Attributes::V03(a) => a.set_type(ty),
            Attributes::V10(a) => a.set_type(ty),
        }
    }

    fn set_subject(&mut self, subject: Option<impl Into<String>>) -> Option<String> {
        match self {
            Attributes::V03(a) => a.set_subject(subject),
            Attributes::V10(a) => a.set_subject(subject),
        }
    }

    fn set_time(&mut self, time: Option<impl Into<DateTime<Utc>>>) -> Option<DateTime<Utc>> {
        match self {
            Attributes::V03(a) => a.set_time(time),
            Attributes::V10(a) => a.set_time(time),
        }
    }

    fn set_datacontenttype(
        &mut self,
        datacontenttype: Option<impl Into<String>>,
    ) -> Option<String> {
        match self {
            Attributes::V03(a) => a.set_datacontenttype(datacontenttype),
            Attributes::V10(a) => a.set_datacontenttype(datacontenttype),
        }
    }

    fn set_dataschema(&mut self, dataschema: Option<impl Into<Url>>) -> Option<Url> {
        match self {
            Attributes::V03(a) => a.set_dataschema(dataschema),
            Attributes::V10(a) => a.set_dataschema(dataschema),
        }
    }
}

impl Attributes {
    pub(crate) fn into_v10(self) -> Self {
        match self {
            Attributes::V03(v03) => Attributes::V10(v03.into_v10()),
            _ => self,
        }
    }
    pub(crate) fn into_v03(self) -> Self {
        match self {
            Attributes::V10(v10) => Attributes::V03(v10.into_v03()),
            _ => self,
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&str, AttributeValue)> {
        match self {
            Attributes::V03(a) => AttributesIter::IterV03(a.into_iter()),
            Attributes::V10(a) => AttributesIter::IterV10(a.into_iter()),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn default_hostname() -> Url {
    Url::parse(
        format!(
            "http://{}",
            hostname::get()
                .ok()
                .and_then(|s| s.into_string().ok())
                .unwrap_or_else(|| "localhost".to_string())
        )
        .as_ref(),
    )
    .unwrap()
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub(crate) fn default_hostname() -> Url {
    use std::str::FromStr;

    Url::from_str(
        web_sys::window()
            .map(|w| w.location().host().ok())
            .flatten()
            .unwrap_or(String::from("http://localhost"))
            .as_str(),
    )
    .unwrap()
}

#[cfg(all(target_arch = "wasm32", target_os = "wasi"))]
pub(crate) fn default_hostname() -> Url {
    use std::str::FromStr;

    Url::from_str("http://localhost").unwrap()
}
