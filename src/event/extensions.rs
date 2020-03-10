use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(untagged)]
/// Possible extension values
pub enum ExtensionValue {
    /// Represents a [`String`] value.
    ///
    /// [`String`]: https://doc.rust-lang.org/std/string/struct.String.html
    String(String),
    Boolean(bool),
    Integer(i64),
    /// Represents a JSON [`Value`].
    ///
    /// [`Value`]: https://docs.serde.rs/serde_json/value/enum.Value.html
    Object(Value),
}

impl ExtensionValue {
    /// Create an [`ExtensionValue`] from a [`Into<String>`].
    ///
    /// # Example
    ///
    /// ```
    /// use cloudevents_rust_sdk::ExtensionValue;
    ///
    /// let value = ExtensionValue::from_string("value");
    /// assert_eq!(value, ExtensionValue::String("value".to_owned()));
    /// ```
    ///
    /// [`Into<String>`]: https://doc.rust-lang.org/std/convert/trait.Into.html
    /// [`ExtensionValue`]: enum.ExtensionValue.html
    pub fn from_string<S>(s: S) -> Self
    where
        S: Into<String>,
    {
        ExtensionValue::String(s.into())
    }

    pub fn from_i64<S>(s: S) -> Self
    where
        S: Into<i64>,
    {
        ExtensionValue::Integer(s.into())
    }

    pub fn from_bool<S>(s: S) -> Self
    where
        S: Into<bool>,
    {
        ExtensionValue::Boolean(s.into())
    }

    pub fn from_json_value<S>(s: S) -> Self
    where
        S: Into<serde_json::Value>,
    {
        ExtensionValue::Object(s.into())
    }
}
