use serde::{Deserialize, Serialize};
use std::convert::{Into, TryFrom};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
/// Possible data values
pub enum Data {
    String(String),
    Binary(Vec<u8>),
    Json(serde_json::Value),
}

impl Data {
    /// Create a [`Data`] from a [`Into<String>`].
    ///
    /// # Example
    ///
    /// ```
    /// use cloudevents::event::Data;
    ///
    /// let value = Data::from_string("value");
    /// assert_eq!(value, Data::from_string("value".to_owned()));
    /// ```
    ///
    /// [`Into<String>`]: https://doc.rust-lang.org/std/convert/trait.Into.html
    /// [`Data`]: enum.Data.html
    pub fn from_string<S>(s: S) -> Self
        where
            S: Into<String>,
    {
        Data::String(s.into())
    }

    // /// Create a [`Data`] from a [`Into<Vec<u8>>`].
    // ///
    // /// # Example
    // ///
    // /// ```
    // /// use cloudevents::event::Data;
    // ///
    // /// let value = Data::from_binary(b"value");
    // /// assert_eq!(value, Data::Binary("dmFsdWU=".into_boxed_bytes().into_vec()));
    // /// ```
    // ///
    // /// [`AsRef<[u8]>`]: https://doc.rust-lang.org/std/convert/trait.AsRef.html
    // /// [`Data`]: enum.Data.html
    // pub fn from_binary<I>(i: I) -> Self
    //     where
    //         I: Into<Vec<u8>>,
    // {
    //     Data::Binary(i.into())
    // }
}

impl Into<Data> for serde_json::Value {
    fn into(self) -> Data {
        Data::Json(self)
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
