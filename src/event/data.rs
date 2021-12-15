use serde_json::Value;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::Formatter;
use std::str;

/// Event [data attribute](https://github.com/cloudevents/spec/blob/master/spec.md#event-data) representation
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Data {
    /// Event has a binary payload
    Binary(Vec<u8>),
    /// Event has a non-json string payload
    String(String),
    /// Event has a json payload
    Json(serde_json::Value),
}

pub(crate) fn is_json_content_type(ct: &str) -> bool {
    ct.starts_with("application/json") || ct.starts_with("text/json") || ct.ends_with("+json")
}

impl From<serde_json::Value> for Data {
    fn from(value: Value) -> Self {
        Data::Json(value)
    }
}

impl From<Vec<u8>> for Data {
    fn from(value: Vec<u8>) -> Self {
        Data::Binary(value)
    }
}

impl From<String> for Data {
    fn from(value: String) -> Self {
        Data::String(value)
    }
}

impl From<&str> for Data {
    fn from(value: &str) -> Self {
        Data::String(String::from(value))
    }
}

impl TryFrom<Data> for serde_json::Value {
    type Error = serde_json::Error;

    fn try_from(value: Data) -> Result<Self, Self::Error> {
        match value {
            Data::Binary(v) => Ok(serde_json::from_slice(&v)?),
            Data::Json(v) => Ok(v),
            Data::String(s) => Ok(serde_json::from_str(&s)?),
        }
    }
}

impl TryFrom<Data> for Vec<u8> {
    type Error = serde_json::Error;

    fn try_from(value: Data) -> Result<Self, Self::Error> {
        match value {
            Data::Binary(v) => Ok(v),
            Data::Json(v) => Ok(serde_json::to_vec(&v)?),
            Data::String(s) => Ok(s.into_bytes()),
        }
    }
}

impl TryFrom<Data> for String {
    type Error = std::string::FromUtf8Error;

    fn try_from(value: Data) -> Result<Self, Self::Error> {
        match value {
            Data::Binary(v) => Ok(String::from_utf8(v)?),
            Data::Json(v) => Ok(v.to_string()),
            Data::String(s) => Ok(s),
        }
    }
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Data::Binary(vec) => write!(f, "Binary data: {:?}", str::from_utf8(vec).unwrap()),
            Data::String(s) => write!(f, "String data: {}", s),
            Data::Json(j) => write!(f, "Json data: {}", j),
        }
    }
}
