use serde::export::Formatter;
use serde_json::Value;
use std::convert::{Into, TryFrom};
use std::fmt;

/// Event [data attribute](https://github.com/cloudevents/spec/blob/master/spec.md#event-data) representation
#[derive(Debug, PartialEq, Clone)]
pub enum Data {
    /// Event has a binary payload
    Binary(Vec<u8>),
    /// Event has a non-json string payload
    String(String),
    /// Event has a json payload
    Json(serde_json::Value),
}

impl Data {
    /// Create a [`Data`] from a [`Into<Vec<u8>>`].
    ///
    /// # Example
    ///
    /// ```
    /// use cloudevents::event::Data;
    ///
    /// let value = Data::from_base64(b"dmFsdWU=").unwrap();
    /// assert_eq!(value, Data::Binary(base64::decode("dmFsdWU=").unwrap()));
    /// ```
    ///
    /// [`AsRef<[u8]>`]: https://doc.rust-lang.org/std/convert/trait.AsRef.html
    /// [`Data`]: enum.Data.html
    pub fn from_base64<I>(i: I) -> Result<Self, base64::DecodeError>
    where
        I: AsRef<[u8]>,
    {
        Ok(base64::decode(&i)?.into())
    }

    pub fn from_binary<I>(content_type: Option<&str>, i: I) -> Result<Self, serde_json::Error>
    where
        I: AsRef<[u8]>,
    {
        let is_json = is_json_content_type(content_type.unwrap_or("application/json"));
        if is_json {
            serde_json::from_slice::<serde_json::Value>(i.as_ref()).map(Data::Json)
        } else {
            Ok(Data::Binary(i.as_ref().to_vec()))
        }
    }
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
            Data::Binary(v) => Ok(serde_json::from_slice(&v)?),
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
            Data::Binary(vec) => write!(f, "Binary data: {:?}", vec),
            Data::String(s) => write!(f, "String data: {}", s),
            Data::Json(j) => write!(f, "Json data: {}", j),
        }
    }
}
