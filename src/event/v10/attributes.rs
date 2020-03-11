use crate::event::{AttributesReader, AttributesWriter, ExtensionValue, SpecVersion};
use chrono::{DateTime, FixedOffset};
use std::collections::HashMap;
use crate::event::attributes::DataAttributesWriter;
use uuid::Uuid;
use hostname::get_hostname;

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

    fn get_source(&self) -> &str {
        &self.source
    }

    fn get_specversion(&self) -> SpecVersion {
        SpecVersion::V10
    }

    fn get_type(&self) -> &str {
        &self.ty
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
    fn set_id<'s, 'event: 's>(&'event mut self, id: impl Into<&'s str>) {
        self.id = id.into().to_owned()
    }

    fn set_source<'s, 'event: 's>(&'event mut self, source: impl Into<&'s str>) {
        self.source = source.into().to_owned()
    }

    fn set_type<'s, 'event: 's>(&'event mut self, ty: impl Into<&'s str>) {
        self.ty = ty.into().to_owned()
    }

    fn set_subject<'s, 'event: 's>(&'event mut self, subject: Option<impl Into<&'s str>>) {
        self.subject = subject.map(Into::into).map(String::from)
    }

    fn set_time(&mut self, time: Option<impl Into<DateTime<FixedOffset>>>) {
        self.time = time.map(Into::into)
    }

    fn set_extension<'s, 'event: 's>(
        &'event mut self,
        extension_name: &'s str,
        extension_value: impl Into<ExtensionValue>,
    ) {
        self.extensions
            .insert(extension_name.to_owned(), extension_value.into());
    }

    fn remove_extension<'s, 'event: 's>(
        &'event mut self,
        extension_name: &'s str,
    ) -> Option<ExtensionValue> {
        self.extensions.remove(extension_name)
    }
}

impl DataAttributesWriter for Attributes {
    fn set_datacontenttype<'s, 'event: 's>(
        &'event mut self,
        datacontenttype: Option<impl Into<&'s str>>,
    ) {
        self.datacontenttype = datacontenttype.map(Into::into).map(String::from)
    }

    fn set_dataschema<'s, 'event: 's>(&'event mut self, dataschema: Option<impl Into<&'s str>>) {
        self.dataschema = dataschema.map(Into::into).map(String::from)
    }
}

impl Default for Attributes {
    fn default() -> Self {
        Attributes {
            id: Uuid::new_v4().to_string(),
            ty: "type".to_string(),
            source: get_hostname().unwrap_or("http://localhost/".to_string()),
            datacontenttype: None,
            dataschema: None,
            subject: None,
            time: None,
            extensions: HashMap::new()
        }
    }
}
