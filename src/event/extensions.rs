use serde::{Deserialize, Serialize, Serializer};
use std::convert::From;
use std::fmt;

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
/// Represents all the possible [CloudEvents extension](https://github.com/cloudevents/spec/blob/master/spec.md#extension-context-attributes) values
pub enum ExtensionValue {
    /// Represents a [`String`] value.
    String(String),
    /// Represents a [`bool`] value.
    Boolean(bool),
    /// Represents an integer [`i64`] value.
    Integer(i64),
}

impl From<&str> for ExtensionValue {
    fn from(s: &str) -> Self {
        ExtensionValue::String(String::from(s))
    }
}

impl From<String> for ExtensionValue {
    fn from(s: String) -> Self {
        ExtensionValue::String(s)
    }
}

impl From<bool> for ExtensionValue {
    fn from(s: bool) -> Self {
        ExtensionValue::Boolean(s)
    }
}

impl From<i64> for ExtensionValue {
    fn from(s: i64) -> Self {
        ExtensionValue::Integer(s)
    }
}

impl ExtensionValue {
    pub fn from_string<S>(s: S) -> Self
    where
        S: Into<String>,
    {
        ExtensionValue::from(s.into())
    }

    pub fn from_i64<S>(s: S) -> Self
    where
        S: Into<i64>,
    {
        ExtensionValue::from(s.into())
    }

    pub fn from_bool<S>(s: S) -> Self
    where
        S: Into<bool>,
    {
        ExtensionValue::from(s.into())
    }
}

impl fmt::Display for ExtensionValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtensionValue::String(s) => f.write_str(s),
            ExtensionValue::Boolean(b) => f.serialize_bool(*b),
            ExtensionValue::Integer(i) => f.serialize_i64(*i),
        }
    }
}
