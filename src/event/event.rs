use super::{
    AttributeValue, Attributes, AttributesReader, AttributesV10, AttributesWriter, Data,
    ExtensionValue, SpecVersion,
};
use chrono::{DateTime, Utc};
use delegate_attr::delegate;
use std::collections::HashMap;
use url::Url;

/// Data structure that represents a [CloudEvent](https://github.com/cloudevents/spec/blob/master/spec.md).
/// It provides methods to get the attributes through [`AttributesReader`]
/// and write them through [`AttributesWriter`].
/// It also provides methods to read and write the [event data](https://github.com/cloudevents/spec/blob/master/spec.md#event-data).
///
/// You can build events using [`super::EventBuilder`]
/// ```
/// use cloudevents::*;
/// use std::convert::TryInto;
///
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// // Create an event using the Default trait
/// let mut e = Event::default();
/// e.set_data(
///     "application/json",
///     serde_json::json!({"hello": "world"})
/// );
///
/// // Print the event id
/// println!("Event id: {}", e.id());
///
/// // Get the event data
/// let data: Option<Data> = e.data().cloned();
/// match data {
///     Some(d) => println!("{}", d),
///     None => println!("No event data")
/// }
/// # Ok(())
/// # }
/// ```
#[derive(PartialEq, Debug, Clone)]
pub struct Event {
    pub(crate) attributes: Attributes,
    pub(crate) data: Option<Data>,
    pub(crate) extensions: HashMap<String, ExtensionValue>,
}

#[delegate(self.attributes)]
impl AttributesReader for Event {
    fn id(&self) -> &str;
    fn source(&self) -> &Url;
    fn specversion(&self) -> SpecVersion;
    fn ty(&self) -> &str;
    fn datacontenttype(&self) -> Option<&str>;
    fn dataschema(&self) -> Option<&Url>;
    fn subject(&self) -> Option<&str>;
    fn time(&self) -> Option<&DateTime<Utc>>;
}

#[delegate(self.attributes)]
impl AttributesWriter for Event {
    fn set_id(&mut self, id: impl Into<String>) -> String;
    fn set_source(&mut self, source: impl Into<Url>) -> Url;
    fn set_type(&mut self, ty: impl Into<String>) -> String;
    fn set_subject(&mut self, subject: Option<impl Into<String>>) -> Option<String>;
    fn set_time(&mut self, time: Option<impl Into<DateTime<Utc>>>) -> Option<DateTime<Utc>>;
    fn set_datacontenttype(&mut self, datacontenttype: Option<impl Into<String>>)
        -> Option<String>;
    fn set_dataschema(&mut self, dataschema: Option<impl Into<Url>>) -> Option<Url>;
}

impl Default for Event {
    fn default() -> Self {
        Event {
            attributes: Attributes::V10(AttributesV10::default()),
            data: None,
            extensions: HashMap::default(),
        }
    }
}

impl Event {
    /// Returns an [`Iterator`] for all the available [CloudEvents Context attributes](https://github.com/cloudevents/spec/blob/master/spec.md#context-attributes) and extensions.
    /// Same as chaining [`Event::iter_attributes()`] and [`Event::iter_extensions()`]
    pub fn iter(&self) -> impl Iterator<Item = (&str, AttributeValue)> {
        self.iter_attributes()
            .chain(self.extensions.iter().map(|(k, v)| (k.as_str(), v.into())))
    }

    /// Returns an [`Iterator`] for all the available [CloudEvents Context attributes](https://github.com/cloudevents/spec/blob/master/spec.md#context-attributes), excluding extensions.
    /// This iterator does not contain the `data` field.
    pub fn iter_attributes(&self) -> impl Iterator<Item = (&str, AttributeValue)> {
        self.attributes.iter()
    }

    /// Get all the [extensions](https://github.com/cloudevents/spec/blob/master/spec.md#extension-context-attributes)
    pub fn iter_extensions(&self) -> impl Iterator<Item = (&str, &ExtensionValue)> {
        self.extensions.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Get `data` from this `Event`
    pub fn data(&self) -> Option<&Data> {
        self.data.as_ref()
    }

    /// Take (`datacontenttype`, `dataschema`, `data`) from this event, leaving these fields empty
    ///
    /// ```
    /// use cloudevents::Event;
    /// use serde_json::json;
    /// use std::convert::Into;
    ///
    /// let mut e = Event::default();
    /// e.set_data("application/json", json!({}));
    ///
    /// let (datacontenttype, dataschema, data) = e.take_data();
    /// ```
    pub fn take_data(&mut self) -> (Option<String>, Option<Url>, Option<Data>) {
        (
            self.attributes.set_datacontenttype(None as Option<String>),
            self.attributes.set_dataschema(None as Option<Url>),
            self.data.take(),
        )
    }

    /// Set `data` into this `Event` with the specified `datacontenttype`.
    /// Returns the previous value of `datacontenttype` and `data`.
    ///
    /// ```
    /// use cloudevents::Event;
    /// use serde_json::json;
    /// use std::convert::Into;
    ///
    /// let mut e = Event::default();
    /// let (old_datacontenttype, old_data) = e.set_data("application/json", json!({}));
    /// ```
    pub fn set_data(
        &mut self,
        datacontenttype: impl Into<String>,
        data: impl Into<Data>,
    ) -> (Option<String>, Option<Data>) {
        (
            self.attributes.set_datacontenttype(Some(datacontenttype)),
            std::mem::replace(&mut self.data, Some(data.into())),
        )
    }

    /// Set `data` into this `Event`, without checking if there is a `datacontenttype`.
    /// Returns the previous value of `data`.
    ///
    /// ```
    /// use cloudevents::Event;
    /// use serde_json::json;
    /// use std::convert::Into;
    ///
    /// let mut e = Event::default();
    /// let old_data = e.set_data_unchecked(json!({}));
    /// ```
    pub fn set_data_unchecked(&mut self, data: impl Into<Data>) -> Option<Data> {
        std::mem::replace(&mut self.data, Some(data.into()))
    }

    /// Get the [extension](https://github.com/cloudevents/spec/blob/master/spec.md#extension-context-attributes) named `extension_name`
    pub fn extension(&self, extension_name: &str) -> Option<&ExtensionValue> {
        self.extensions.get(extension_name)
    }

    /// Set the [extension](https://github.com/cloudevents/spec/blob/master/spec.md#extension-context-attributes) named `extension_name` with `extension_value`
    pub fn set_extension<'name, 'event: 'name>(
        &'event mut self,
        extension_name: &'name str,
        extension_value: impl Into<ExtensionValue>,
    ) {
        self.extensions
            .insert(extension_name.to_owned(), extension_value.into());
    }

    /// Remove the [extension](https://github.com/cloudevents/spec/blob/master/spec.md#extension-context-attributes) named `extension_name`
    pub fn remove_extension<'name, 'event: 'name>(
        &'event mut self,
        extension_name: &'name str,
    ) -> Option<ExtensionValue> {
        self.extensions.remove(extension_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_data() {
        let mut e = Event::default();
        e.set_data(
            "application/json",
            serde_json::json!({
                "hello": "world"
            }),
        );

        let (datacontenttype, dataschema, data) = e.take_data();

        assert!(datacontenttype.is_some());
        assert!(dataschema.is_none());
        assert!(data.is_some());

        assert!(e.data().is_none());
        assert!(e.dataschema().is_none());
        assert!(e.datacontenttype().is_none());
    }

    #[test]
    fn set_id() {
        let mut e = Event::default();
        e.set_id("001");

        assert_eq!(e.set_id("002"), String::from("001"));
        assert_eq!(e.id(), "002")
    }

    #[test]
    fn iter() {
        let mut e = Event::default();
        e.set_extension("aaa", "bbb");
        e.set_data(
            "application/json",
            serde_json::json!({
                "hello": "world"
            }),
        );

        let mut v: HashMap<&str, AttributeValue> = e.iter().collect();

        assert_eq!(
            v.remove("specversion"),
            Some(AttributeValue::SpecVersion(SpecVersion::V10))
        );
        assert_eq!(v.remove("aaa"), Some(AttributeValue::String("bbb")))
    }
}
