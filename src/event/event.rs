use super::{
    Attributes, AttributesReader, AttributesV10, AttributesWriter, Data, ExtensionValue,
    SpecVersion,
};
use crate::event::attributes::DataAttributesWriter;
use chrono::{DateTime, Utc};
use delegate::delegate;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

/// Data structure that represents a [CloudEvent](https://github.com/cloudevents/spec/blob/master/spec.md).
/// It provides methods to get the attributes through [`AttributesReader`]
/// and write them through [`AttributesWriter`].
/// It also provides methods to read and write the [event data](https://github.com/cloudevents/spec/blob/master/spec.md#event-data).
///
/// You can build events using [`EventBuilder`]
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
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "data_base64")]
    #[serde(alias = "data")]
    #[serde(flatten)]
    pub data: Option<Data>,
    #[serde(flatten)]
    pub attributes: Attributes,
}

impl AttributesReader for Event {
    delegate! {
        to self.attributes {
            fn get_id(&self) -> &str;
            fn get_source(&self) -> &str;
            fn get_specversion(&self) -> SpecVersion;
            fn get_type(&self) -> &str;
            fn get_datacontenttype(&self) -> Option<&str>;
            fn get_dataschema(&self) -> Option<&str>;
            fn get_subject(&self) -> Option<&str>;
            fn get_time(&self) -> Option<&DateTime<Utc>>;
            fn get_extension(&self, extension_name: &str) -> Option<&ExtensionValue>;
            fn get_extensions(&self) -> Vec<(&str, &ExtensionValue)>;
        }
    }
}

impl AttributesWriter for Event {
    delegate! {
        to self.attributes {
            fn set_id(&mut self, id: impl Into<String>);
            fn set_source(&mut self, source: impl Into<String>);
            fn set_type(&mut self, ty: impl Into<String>);
            fn set_subject(&mut self, subject: Option<impl Into<String>>);
            fn set_time(&mut self, time: Option<impl Into<DateTime<Utc>>>);
            fn set_extension<'name, 'event: 'name>(
                &'event mut self,
                extension_name: &'name str,
                extension_value: impl Into<ExtensionValue>,
            );
            fn remove_extension<'name, 'event: 'name>(
                &'event mut self,
                extension_name: &'name str,
            ) -> Option<ExtensionValue>;
        }
    }
}

impl Default for Event {
    fn default() -> Self {
        Event {
            attributes: Attributes::V10(AttributesV10::default()),
            data: None,
        }
    }
}

impl Event {
    pub fn remove_data(&mut self) {
        self.data = None;
        self.attributes.set_dataschema(None as Option<String>);
        self.attributes.set_datacontenttype(None as Option<String>);
    }

    /// Write data into the `Event` with the specified `datacontenttype`.
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
        self.attributes.set_dataschema(None as Option<&str>);
        self.data = Some(data.into());
    }

    /// Write data into the `Event` with the specified `datacontenttype` and `dataschema`.
    ///
    /// ```
    /// use cloudevents::Event;
    /// use serde_json::json;
    /// use std::convert::Into;
    ///
    /// let mut e = Event::default();
    /// e.write_data_with_schema("application/json", "http://myapplication.com/schema", json!({}))
    /// ```
    pub fn write_data_with_schema(
        &mut self,
        datacontenttype: impl Into<String>,
        dataschema: impl Into<String>,
        data: impl Into<Data>,
    ) {
        self.attributes.set_datacontenttype(Some(datacontenttype));
        self.attributes.set_dataschema(Some(dataschema));
        self.data = Some(data.into());
    }

    pub fn get_data<T: Sized + From<Data>>(&self) -> Option<T> {
        match self.data.as_ref() {
            Some(d) => Some(T::from(d.clone())),
            None => None,
        }
    }

    pub fn try_get_data<T: Sized + TryFrom<Data>>(&self) -> Result<Option<T>, T::Error> {
        match self.data.as_ref() {
            Some(d) => Some(T::try_from(d.clone())),
            None => None,
        }
        .transpose()
    }

    pub fn into_data<T: Sized + TryFrom<Data>>(self) -> Result<Option<T>, T::Error> {
        match self.data {
            Some(d) => Some(T::try_from(d)),
            None => None,
        }
        .transpose()
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
            "http://localhost:8080/schema",
            expected_data.clone(),
        );

        let data: serde_json::Value = e.try_get_data().unwrap().unwrap();
        assert_eq!(expected_data, data);
        assert_eq!("application/json", e.get_datacontenttype().unwrap());
        assert_eq!("http://localhost:8080/schema", e.get_dataschema().unwrap())
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
