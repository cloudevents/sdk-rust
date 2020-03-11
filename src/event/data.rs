use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::TryInto;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
/// Possible data values
pub enum Data {
    String(String),
    Binary(Vec<u8>),
    Object(Value),
}
//
// impl Data {
//     /// Create a [`Data`] from a [`Into<String>`].
//     ///
//     /// # Example
//     ///
//     /// ```
//     /// use cloudevents::Data;
//     ///
//     /// let value = Data::from_string("value");
//     /// assert_eq!(value, Data::StringOrBinary("value".to_owned()));
//     /// ```
//     ///
//     /// [`Into<String>`]: https://doc.rust-lang.org/std/convert/trait.Into.html
//     /// [`Data`]: enum.Data.html
//     pub fn from_string<S>(s: S) -> Self
//         where
//             S: Into<String>,
//     {
//         Data::StringOrBinary(s.into())
//     }
//
//     /// Create a [`Data`] from a [`AsRef<[u8]>`].
//     ///
//     /// # Example
//     ///
//     /// ```
//     /// use cloudevents::Data;
//     ///
//     /// let value = Data::from_binary(b"value");
//     /// assert_eq!(value, Data::StringOrBinary("dmFsdWU=".to_owned()));
//     /// ```
//     ///
//     /// [`AsRef<[u8]>`]: https://doc.rust-lang.org/std/convert/trait.AsRef.html
//     /// [`Data`]: enum.Data.html
//     pub fn from_binary<I>(i: I) -> Self
//         where
//             I: AsRef<[u8]>,
//     {
//         Data::StringOrBinary(base64::encode(&i))
//     }
//
//     /// Create a [`Data`] from a [`Serialize`] object.
//     ///
//     /// # Example
//     ///
//     /// ```
//     /// use cloudevents::Data;
//     /// use serde_json::Value;
//     /// use std::error::Error;
//     ///
//     /// fn main() -> Result<(), Box<Error>> {
//     ///     let value = Data::from_serializable("value")?;
//     ///     assert_eq!(value, Data::Object(Value::String("value".to_owned())));
//     ///     Ok(())
//     /// }
//     /// ```
//     ///
//     /// [`Serialize`]: https://docs.serde.rs/serde/ser/trait.Serialize.html
//     /// [`Data`]: enum.Data.html
//     pub fn from_serializable<T>(v: T) -> Result<Self, Error>
//         where
//             T: Serialize,
//     {
//         Ok(Data::Object(serde_json::to_value(v)?))
//     }
// }

impl Into<Data> for String {
    fn into(self) -> Data {
        Data::String(self)
    }
}

// TODO Define an error here?
impl TryInto<String> for Data {
    type Error = ();

    fn try_into(self) -> Result<String, Self::Error> {
        match self {
            Data::String(s) => Ok(s),
            Data::Binary(v) => String::from_utf8(v).map_err(|_e| ()),
            _ => Err(()),
        }
    }
}
