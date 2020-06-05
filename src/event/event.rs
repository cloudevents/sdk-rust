use super::{
    AttributeValue, Attributes, AttributesIter, AttributesReader, AttributesV10, AttributesWriter,
    Data, ExtensionValue, SpecVersion,
};
use crate::event::attributes::DataAttributesWriter;
use chrono::{DateTime, Utc};
use delegate::delegate;
use std::collections::HashMap;
use std::convert::TryFrom;
use url::Url;

/// Data structure that represents a [CloudEvent](https://github.com/cloudevents/spec/blob/master/spec.md).
/// It provides methods to get the attributes through [`AttributesReader`]
/// and write them through [`AttributesWriter`].
/// It also provides methods to read and write the [event data](https://github.com/cloudevents/spec/blob/master/spec.md#event-data).
///
/// You can build events using [`super::EventBuilder`]
/// ```
/// use cloudevents::Event;
/// use cloudevents::event::AttributesReader;
///
/// // Create an event using the Default trait
/// let mut e = Event::default();
/// e.write_data(
///     "application/json",
///     serde_json::json!({"hello": "world"})
/// );
///
/// // Print the event id
/// println!("Event id: {}", e.get_id());
///
/// // Get the event data
/// let data: serde_json::Value = e.try_get_data().unwrap().unwrap();
/// println!("Event data: {}", data)
/// ```
#[derive(PartialEq, Debug, Clone)]
pub struct Event {
    pub(crate) attributes: Attributes,
    pub(crate) data: Option<Data>,
    pub(crate) extensions: HashMap<String, ExtensionValue>,
}

impl AttributesReader for Event {
    delegate! {
        to self.attributes {
            fn get_id(&self) -> &str;
            fn get_source(&self) -> &Url;
            fn get_specversion(&self) -> SpecVersion;
            fn get_type(&self) -> &str;
            fn get_datacontenttype(&self) -> Option<&str>;
            fn get_dataschema(&self) -> Option<&Url>;
            fn get_subject(&self) -> Option<&str>;
            fn get_time(&self) -> Option<&DateTime<Utc>>;
        }
    }
}

impl AttributesWriter for Event {
    delegate! {
        to self.attributes {
            fn set_id(&mut self, id: impl Into<String>);
            fn set_source(&mut self, source: impl Into<Url>);
            fn set_type(&mut self, ty: impl Into<String>);
            fn set_subject(&mut self, subject: Option<impl Into<String>>);
            fn set_time(&mut self, time: Option<impl Into<DateTime<Utc>>>);
        }
    }
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
    /// Returns an `Iterator` for `Attributes`
    pub fn attributes_iter(&self) -> impl Iterator<Item = (&str, AttributeValue<'_>)> {
        match &self.attributes {
            Attributes::V03(a) => AttributesIter::IterV03(a.into_iter()),
            Attributes::V10(a) => AttributesIter::IterV10(a.into_iter()),
        }
    }
    /// Remove `data`, `dataschema` and `datacontenttype` from this `Event`
    pub fn remove_data(&mut self) {
        self.data = None;
        self.attributes.set_dataschema(None as Option<Url>);
        self.attributes.set_datacontenttype(None as Option<String>);
    }

    /// Write `data` into this `Event` with the specified `datacontenttype`.
    ///
    /// ```
    /// use cloudevents::Event;
    /// use serde_json::json;
    /// use std::convert::Into;
    ///
    /// let mut e = Event::default();
    /// e.write_data("application/json", json!({}))
    /// ```
    pub fn write_data(&mut self, datacontenttype: impl Into<String>, data: impl Into<Data>) {
        self.attributes.set_datacontenttype(Some(datacontenttype));
        self.attributes.set_dataschema(None as Option<Url>);
        self.data = Some(data.into());
    }

    /// Write `data` into this `Event` with the specified `datacontenttype` and `dataschema`.
    ///
    /// ```
    /// use cloudevents::Event;
    /// use serde_json::json;
    /// use std::convert::Into;
    /// use url::Url;
    ///
    /// let mut e = Event::default();
    /// e.write_data_with_schema(
    ///     "application/json",
    ///     Url::parse("http://myapplication.com/schema").unwrap(),
    ///     json!({})
    /// )
    /// ```
    pub fn write_data_with_schema(
        &mut self,
        datacontenttype: impl Into<String>,
        dataschema: impl Into<Url>,
        data: impl Into<Data>,
    ) {
        self.attributes.set_datacontenttype(Some(datacontenttype));
        self.attributes.set_dataschema(Some(dataschema));
        self.data = Some(data.into());
    }

    /// Get `data` from this `Event`
    pub fn get_data<T: Sized + From<Data>>(&self) -> Option<T> {
        match self.data.as_ref() {
            Some(d) => Some(T::from(d.clone())),
            None => None,
        }
    }

    /// Try to get `data` from this `Event`
    pub fn try_get_data<T: Sized + TryFrom<Data>>(&self) -> Result<Option<T>, T::Error> {
        match self.data.as_ref() {
            Some(d) => Some(T::try_from(d.clone())),
            None => None,
        }
        .transpose()
    }

    /// Transform this `Event` into the content of `data`
    pub fn into_data<T: Sized + TryFrom<Data>>(self) -> Result<Option<T>, T::Error> {
        match self.data {
            Some(d) => Some(T::try_from(d)),
            None => None,
        }
        .transpose()
    }

    /// Get the [extension](https://github.com/cloudevents/spec/blob/master/spec.md#extension-context-attributes) named `extension_name`
    pub fn get_extension(&self, extension_name: &str) -> Option<&ExtensionValue> {
        self.extensions.get(extension_name)
    }

    /// Get all the [extensions](https://github.com/cloudevents/spec/blob/master/spec.md#extension-context-attributes)
    pub fn get_extensions(&self) -> Vec<(&str, &ExtensionValue)> {
        self.extensions
            .iter()
            .map(|(k, v)| (k.as_str(), v))
            .collect()
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
    fn try_get_data_json() {
        let expected_data = serde_json::json!({
            "hello": "world"
        });

        let mut e = Event::default();
        e.write_data_with_schema(
            "application/json",
            Url::parse("http://localhost:8080/schema").unwrap(),
            expected_data.clone(),
        );

        let data: serde_json::Value = e.try_get_data().unwrap().unwrap();
        assert_eq!(expected_data, data);
        assert_eq!("application/json", e.get_datacontenttype().unwrap());
        assert_eq!(
            &Url::parse("http://localhost:8080/schema").unwrap(),
            e.get_dataschema().unwrap()
        )
    }

    #[test]
    fn remove_data() {
        let mut e = Event::default();
        e.write_data(
            "application/json",
            serde_json::json!({
                "hello": "world"
            }),
        );

        e.remove_data();

        assert!(e.try_get_data::<serde_json::Value>().unwrap().is_none());
        assert!(e.get_dataschema().is_none());
        assert!(e.get_datacontenttype().is_none());
    }
}
