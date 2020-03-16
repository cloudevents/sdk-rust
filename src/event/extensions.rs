use std::convert::From;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
/// Represents all the possible [CloudEvents extension](https://github.com/cloudevents/spec/blob/master/spec.md#extension-context-attributes) values
pub enum ExtensionValue {
    /// Represents a [`String`](std::string::String) value.
    String(String),
    /// Represents a [`bool`](bool) value.
    Boolean(bool),
    /// Represents an integer [`i64`](i64) value.
    Integer(i64)
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
