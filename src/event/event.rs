use super::{Attributes, AttributesReader, AttributesWriter, Data, ExtensionValue, SpecVersion, AttributesV10};
use chrono::{DateTime, FixedOffset};
use delegate::delegate;
use std::convert::{TryFrom};
use crate::event::attributes::DataAttributesWriter;

/// Data structure that represents a [CloudEvent](https://github.com/cloudevents/spec/blob/master/spec.md).
/// It provides methods to get the attributes through [`AttributesReader`]
/// and write them through [`AttributesWriter`].
/// It also provides methods to read and write the [event data](https://github.com/cloudevents/spec/blob/master/spec.md#event-data)
/// ```
/// use cloudevents::Event;
/// use cloudevents::event::AttributesReader;
///
/// // Create an event using the Default trait
/// let mut e = Event::default();
/// e.write_data(
///     "application/json",
///     None,
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
pub struct Event {
    pub attributes: Attributes,
    pub data: Option<Data>,
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
            fn get_time(&self) -> Option<&DateTime<FixedOffset>>;
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
            fn set_time(&mut self, time: Option<impl Into<DateTime<FixedOffset>>>);
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
            data: None
        }
    }
}

impl Event {
    pub fn remove_data(&mut self) {
        self.data = None;
    }

    /// Write data into the `Event`. You must provide a `content_type` and you can optionally provide a `schema`.
    ///
    /// ```
    /// use cloudevents::Event;
    /// use serde_json::json;
    /// use std::convert::Into;
    ///
    /// let mut e = Event::default();
    /// e.write_data("application/json", None, json!({}))
    /// ```
    pub fn write_data<S: Into<String>, D: Into<Data>>(&mut self, content_type: S, schema: Option<S>, value: D) {
        self.attributes.set_datacontenttype(Some(content_type));
        self.attributes.set_dataschema(schema);
        self.data = Some(value.into());
    }

    pub fn get_data<T: Sized + From<Data>>(
        &self,
    ) -> Option<T> {
        match self.data.as_ref() {
            Some(d) => Some(T::from(d.clone())),
            None => None,
        }
    }

    pub fn try_get_data<T: Sized + TryFrom<Data, Error = E>, E: std::error::Error>(
        &self,
    ) -> Option<Result<T, E>> {
        match self.data.as_ref() {
            Some(d) => Some(T::try_from(d.clone())),
            None => None,
        }
    }

    pub fn into_data<T: Sized + TryFrom<Data, Error = E>, E: std::error::Error>(
        self,
    ) -> Option<Result<T, E>> {
        match self.data {
            Some(d) => Some(T::try_from(d)),
            None => None,
        }
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
         e.write_data(
             "application/json",
             None,
             expected_data.clone()
         );


        let data: serde_json::Value = e.try_get_data().unwrap().unwrap();
        assert_eq!(expected_data, data);
        assert_eq!("application/json", e.get_datacontenttype().unwrap())
    }

}
