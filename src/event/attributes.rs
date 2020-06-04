use super::{
    AttributesIntoIteratorV03, AttributesIntoIteratorV10, AttributesV03, AttributesV10, SpecVersion,
};
use chrono::{DateTime, Utc};
use std::fmt;
use url::Url;

#[derive(Debug, PartialEq)]
pub enum AttributeValue<'a> {
    SpecVersion(SpecVersion),
    String(&'a str),
    URI(&'a Url),
    URIRef(&'a Url),
    Time(&'a DateTime<Utc>),
}

impl fmt::Display for AttributeValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttributeValue::SpecVersion(s) => s.fmt(f),
            AttributeValue::String(s) => f.write_str(s),
            AttributeValue::URI(s) => f.write_str(&s.as_str()),
            AttributeValue::URIRef(s) => f.write_str(&s.as_str()),
            AttributeValue::Time(s) => f.write_str(&s.to_rfc3339()),
        }
    }
}

/// Trait to get [CloudEvents Context attributes](https://github.com/cloudevents/spec/blob/master/spec.md#context-attributes).
pub trait AttributesReader {
    /// Get the [id](https://github.com/cloudevents/spec/blob/master/spec.md#id).
    fn get_id(&self) -> &str;
    /// Get the [source](https://github.com/cloudevents/spec/blob/master/spec.md#source-1).
    fn get_source(&self) -> &Url;
    /// Get the [specversion](https://github.com/cloudevents/spec/blob/master/spec.md#specversion).
    fn get_specversion(&self) -> SpecVersion;
    /// Get the [type](https://github.com/cloudevents/spec/blob/master/spec.md#type).
    fn get_type(&self) -> &str;
    /// Get the [datacontenttype](https://github.com/cloudevents/spec/blob/master/spec.md#datacontenttype).
    fn get_datacontenttype(&self) -> Option<&str>;
    /// Get the [dataschema](https://github.com/cloudevents/spec/blob/master/spec.md#dataschema).
    fn get_dataschema(&self) -> Option<&Url>;
    /// Get the [subject](https://github.com/cloudevents/spec/blob/master/spec.md#subject).
    fn get_subject(&self) -> Option<&str>;
    /// Get the [time](https://github.com/cloudevents/spec/blob/master/spec.md#time).
    fn get_time(&self) -> Option<&DateTime<Utc>>;
}

/// Trait to set [CloudEvents Context attributes](https://github.com/cloudevents/spec/blob/master/spec.md#context-attributes).
pub trait AttributesWriter {
    /// Set the [id](https://github.com/cloudevents/spec/blob/master/spec.md#id).
    fn set_id(&mut self, id: impl Into<String>);
    /// Set the [source](https://github.com/cloudevents/spec/blob/master/spec.md#source-1).
    fn set_source(&mut self, source: impl Into<Url>);
    /// Set the [type](https://github.com/cloudevents/spec/blob/master/spec.md#type).
    fn set_type(&mut self, ty: impl Into<String>);
    /// Set the [subject](https://github.com/cloudevents/spec/blob/master/spec.md#subject).
    fn set_subject(&mut self, subject: Option<impl Into<String>>);
    /// Set the [time](https://github.com/cloudevents/spec/blob/master/spec.md#time).
    fn set_time(&mut self, time: Option<impl Into<DateTime<Utc>>>);
}

pub(crate) trait AttributesConverter {
    fn into_v03(self) -> AttributesV03;
    fn into_v10(self) -> AttributesV10;
}

pub(crate) trait DataAttributesWriter {
    fn set_datacontenttype(&mut self, datacontenttype: Option<impl Into<String>>);
    fn set_dataschema(&mut self, dataschema: Option<impl Into<Url>>);
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum AttributesIter<'a> {
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
#[derive(PartialEq, Debug, Clone)]
pub enum Attributes {
    V03(AttributesV03),
    V10(AttributesV10),
}

impl AttributesReader for Attributes {
    fn get_id(&self) -> &str {
        match self {
            Attributes::V03(a) => a.get_id(),
            Attributes::V10(a) => a.get_id(),
        }
    }

    fn get_source(&self) -> &Url {
        match self {
            Attributes::V03(a) => a.get_source(),
            Attributes::V10(a) => a.get_source(),
        }
    }

    fn get_specversion(&self) -> SpecVersion {
        match self {
            Attributes::V03(a) => a.get_specversion(),
            Attributes::V10(a) => a.get_specversion(),
        }
    }

    fn get_type(&self) -> &str {
        match self {
            Attributes::V03(a) => a.get_type(),
            Attributes::V10(a) => a.get_type(),
        }
    }

    fn get_datacontenttype(&self) -> Option<&str> {
        match self {
            Attributes::V03(a) => a.get_datacontenttype(),
            Attributes::V10(a) => a.get_datacontenttype(),
        }
    }

    fn get_dataschema(&self) -> Option<&Url> {
        match self {
            Attributes::V03(a) => a.get_dataschema(),
            Attributes::V10(a) => a.get_dataschema(),
        }
    }

    fn get_subject(&self) -> Option<&str> {
        match self {
            Attributes::V03(a) => a.get_subject(),
            Attributes::V10(a) => a.get_subject(),
        }
    }

    fn get_time(&self) -> Option<&DateTime<Utc>> {
        match self {
            Attributes::V03(a) => a.get_time(),
            Attributes::V10(a) => a.get_time(),
        }
    }
}

impl AttributesWriter for Attributes {
    fn set_id(&mut self, id: impl Into<String>) {
        match self {
            Attributes::V03(a) => a.set_id(id),
            Attributes::V10(a) => a.set_id(id),
        }
    }

    fn set_source(&mut self, source: impl Into<Url>) {
        match self {
            Attributes::V03(a) => a.set_source(source),
            Attributes::V10(a) => a.set_source(source),
        }
    }

    fn set_type(&mut self, ty: impl Into<String>) {
        match self {
            Attributes::V03(a) => a.set_type(ty),
            Attributes::V10(a) => a.set_type(ty),
        }
    }

    fn set_subject(&mut self, subject: Option<impl Into<String>>) {
        match self {
            Attributes::V03(a) => a.set_subject(subject),
            Attributes::V10(a) => a.set_subject(subject),
        }
    }

    fn set_time(&mut self, time: Option<impl Into<DateTime<Utc>>>) {
        match self {
            Attributes::V03(a) => a.set_time(time),
            Attributes::V10(a) => a.set_time(time),
        }
    }
}

impl DataAttributesWriter for Attributes {
    fn set_datacontenttype(&mut self, datacontenttype: Option<impl Into<String>>) {
        match self {
            Attributes::V03(a) => a.set_datacontenttype(datacontenttype),
            Attributes::V10(a) => a.set_datacontenttype(datacontenttype),
        }
    }

    fn set_dataschema(&mut self, dataschema: Option<impl Into<Url>>) {
        match self {
            Attributes::V03(a) => a.set_dataschema(dataschema),
            Attributes::V10(a) => a.set_dataschema(dataschema),
        }
    }
}

impl Attributes {
    pub fn into_v10(self) -> Self {
        match self {
            Attributes::V03(v03) => Attributes::V10(v03.into_v10()),
            _ => self,
        }
    }
    pub fn into_v03(self) -> Self {
        match self {
            Attributes::V10(v10) => Attributes::V03(v10.into_v03()),
            _ => self,
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
                .map(|s| s.into_string().ok())
                .flatten()
                .unwrap_or(String::from("localhost".to_string()))
        )
        .as_ref(),
    )
    .unwrap()
}

#[cfg(target_arch = "wasm32")]
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
