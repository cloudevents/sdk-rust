use super::{Attributes, AttributesReader, AttributesWriter, Data, ExtensionValue, SpecVersion};
use chrono::{DateTime, FixedOffset};
use delegate::delegate;
use std::convert::{TryFrom, TryInto};
use crate::event::attributes::DataAttributesWriter;

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
            fn set_id<'s, 'event: 's>(&'event mut self, id: impl Into<&'s str>);
            fn set_source<'s, 'event: 's>(&'event mut self, source: impl Into<&'s str>);
            fn set_type<'s, 'event: 's>(&'event mut self, ty: impl Into<&'s str>);
            fn set_subject<'s, 'event: 's>(&'event mut self, subject: Option<impl Into<&'s str>>);
            fn set_time(&mut self, time: Option<impl Into<DateTime<FixedOffset>>>);
            fn set_extension<'s, 'event: 's>(
                &'event mut self,
                extension_name: &'s str,
                extension_value: impl Into<ExtensionValue>,
            );
            fn remove_extension<'s, 'event: 's>(
                &'event mut self,
                extension_name: &'s str,
            ) -> Option<ExtensionValue>;
        }
    }
}

impl Default for Event {
    fn default() -> Self {
        Event {
            attributes: Attributes::V10(crate::AttributesV10::default()),
            data: None
        }
    }
}

impl Event {
    pub fn remove_data(&mut self) {
        self.data = None;
    }

    ///
    /// ```
    /// use cloudevents::Event;
    ///
    /// let mut e = Event::default();
    /// e.write_data("application/json", None, json!{})
    /// ```
    pub fn write_data<'s, 'event: 's>(&'event mut self, content_type: impl Into<&'s str>, schema: Option<impl Into<&'s str>>, value: impl Into<Data>) {
        self.attributes.set_datacontenttype::<'s, 'event>(Some(content_type));
        self.attributes.set_dataschema::<'s, 'event>(schema);
        self.data = Some(value.into());
    }

    pub fn try_write_data<'s, 'event: 's, E: std::error::Error>(
        &'event mut self,
        content_type: impl Into<&'s str>, schema: Option<impl Into<&'s str>>,
        value: impl TryInto<Data, Error = E>,
    ) -> Result<(), E> {
        Ok(self.write_data(content_type, schema, value.try_into()?))
    }

    pub fn get_data<T: Sized + TryFrom<Data, Error = E>, E: std::error::Error>(
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
