use crate::event::{AttributesReader, AttributesWriter, ExtensionValue, SpecVersion};
use chrono::{DateTime, FixedOffset};
use std::collections::HashMap;

pub struct Attributes {
    id: String,
    ty: String,
    source: String,
    datacontenttype: Option<String>,
    dataschema: Option<String>,
    subject: Option<String>,
    time: Option<DateTime<FixedOffset>>,
    extensions: HashMap<String, ExtensionValue>,
}

impl AttributesReader for Attributes {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn get_type(&self) -> &str {
        &self.ty
    }

    fn get_source(&self) -> &str {
        &self.source
    }

    fn get_specversion(&self) -> SpecVersion {
        SpecVersion::V10
    }

    fn get_datacontenttype(&self) -> Option<&str> {
        match self.datacontenttype.as_ref() {
            Some(s) => Some(&s),
            None => None,
        }
    }

    fn get_dataschema(&self) -> Option<&str> {
        match self.dataschema.as_ref() {
            Some(s) => Some(&s),
            None => None,
        }
    }

    fn get_subject(&self) -> Option<&str> {
        match self.subject.as_ref() {
            Some(s) => Some(&s),
            None => None,
        }
    }

    fn get_time(&self) -> Option<&DateTime<FixedOffset>> {
        self.time.as_ref()
    }

    fn get_extension(&self, extension_name: &str) -> Option<&ExtensionValue> {
        self.extensions.get(extension_name)
    }

    fn get_extensions(&self) -> Vec<(&str, &ExtensionValue)> {
        self.extensions
            .iter()
            .map(|(k, v)| (k.as_str(), v))
            .collect()
    }
}

impl AttributesWriter for Attributes {
    fn set_id<'event>(&'event mut self, id: impl Into<&'event str>) {
        self.id = id.into().to_owned()
    }

    fn set_type<'event>(&'event mut self, ty: impl Into<&'event str>) {
        self.ty = ty.into().to_owned()
    }

    fn set_source<'event>(&'event mut self, source: impl Into<&'event str>) {
        self.source = source.into().to_owned()
    }

    fn set_datacontenttype<'event>(
        &'event mut self,
        datacontenttype: Option<impl Into<&'event str>>,
    ) {
        self.datacontenttype = datacontenttype.map(Into::into).map(String::from)
    }

    fn set_dataschema<'event>(&'event mut self, dataschema: Option<impl Into<&'event str>>) {
        self.dataschema = dataschema.map(Into::into).map(String::from)
    }

    fn set_subject<'event>(&'event mut self, subject: Option<impl Into<&'event str>>) {
        self.subject = subject.map(Into::into).map(String::from)
    }

    fn set_time<'event>(&'event mut self, time: Option<impl Into<DateTime<FixedOffset>>>) {
        self.time = time.map(Into::into)
    }

    fn set_extension<'event>(
        &'event mut self,
        extension_name: &'event str,
        extension_value: impl Into<ExtensionValue>,
    ) {
        self.extensions
            .insert(extension_name.to_owned(), extension_value.into());
    }

    fn remove_extension<'event>(
        &'event mut self,
        extension_name: &'event str,
    ) -> Option<ExtensionValue> {
        self.extensions.remove(extension_name)
    }
}
