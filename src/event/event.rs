use super::{Attributes, AttributesReader, AttributesWriter, Data, ExtensionValue, SpecVersion};
use chrono::{DateTime, FixedOffset};
use delegate::delegate;
use std::convert::{TryFrom, TryInto};

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
            fn set_id<'event>(&'event mut self, id: impl Into<&'event str>);
            fn set_source<'event>(&'event mut self, source: impl Into<&'event str>);
            fn set_type<'event>(&'event mut self, ty: impl Into<&'event str>);
            fn set_datacontenttype<'event>(
                &'event mut self,
                datacontenttype: Option<impl Into<&'event str>>,
            );
            fn set_dataschema<'event>(&'event mut self, dataschema: Option<impl Into<&'event str>>);
            fn set_subject<'event>(&'event mut self, subject: Option<impl Into<&'event str>>);
            fn set_time<'event>(&'event mut self, time: Option<impl Into<DateTime<FixedOffset>>>);
            fn set_extension<'event>(
                &'event mut self,
                extension_name: &'event str,
                extension_value: impl Into<ExtensionValue>,
            );
            fn remove_extension<'event>(&'event mut self, extension_name: &'event str) -> Option<ExtensionValue>;
        }
    }
}

impl Event {
    fn remove_data(&mut self) {
        self.data = None;
    }

    fn write_data(&mut self, v: impl Into<Data>) {
        self.data = Some(v.into());
    }

    fn try_write_data<E: std::error::Error>(
        &mut self,
        v: impl TryInto<Data, Error = E>,
    ) -> Result<(), E> {
        self.data = Some(v.try_into()?);
        Ok(())
    }

    fn get_data<T: Sized + TryFrom<Data, Error = E>, E: std::error::Error>(
        &self,
    ) -> Option<Result<T, E>> {
        match self.data.as_ref() {
            Some(d) => Some(T::try_from(d.clone())),
            None => None,
        }
    }

    fn into_data<T: Sized + TryFrom<Data, Error = E>, E: std::error::Error>(
        self,
    ) -> Option<Result<T, E>> {
        match self.data {
            Some(d) => Some(T::try_from(d)),
            None => None,
        }
    }
}
