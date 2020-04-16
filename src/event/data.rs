use std::convert::{Into, TryFrom};

#[derive(Debug, PartialEq, Clone)]
/// Possible data values
pub enum Data {
    String(String),
    Binary(Vec<u8>),
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
}

impl Into<Data> for serde_json::Value {
    fn into(self) -> Data {
        Data::Json(self)
    }
}

impl Into<Data> for Vec<u8> {
    fn into(self) -> Data {
        Data::Binary(self)
    }
}

impl Into<Data> for String {
    fn into(self) -> Data {
        Data::String(self)
    }
}

impl TryFrom<Data> for serde_json::Value {
    type Error = serde_json::Error;

    fn try_from(value: Data) -> Result<Self, Self::Error> {
        match value {
            Data::String(s) => Ok(serde_json::from_str(&s)?),
            Data::Binary(v) => Ok(serde_json::from_slice(&v)?),
            Data::Json(v) => Ok(v),
        }
    }
}

impl TryFrom<Data> for String {
    type Error = std::string::FromUtf8Error;

    fn try_from(value: Data) -> Result<Self, Self::Error> {
        match value {
            Data::String(s) => Ok(s),
            Data::Binary(v) => Ok(String::from_utf8(v)?),
            Data::Json(s) => Ok(s.to_string()),
        }
    }
}
