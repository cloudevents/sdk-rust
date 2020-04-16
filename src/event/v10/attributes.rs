use crate::event::attributes::{AttributesConverter, AttributeValue, DataAttributesWriter};
use crate::event::{AttributesReader, AttributesV03, AttributesWriter, SpecVersion};
use chrono::{DateTime, Utc};
use hostname::get_hostname;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(PartialEq, Debug, Clone)]
pub struct Attributes {
    id: String,
    ty: String,
    source: String,
    datacontenttype: Option<String>,
    dataschema: Option<String>,
    subject: Option<String>,
    time: Option<DateTime<Utc>>,
    extensions: HashMap<String, ExtensionValue>,
}

impl<'a> IntoIterator for &'a Attributes {
    type Item = (&'a str, AttributeValue<'a>);
    type IntoIter = AttributesIntoIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        AttributesIntoIterator {
            attributes: self,
            index: 0,
        }
    }
}

struct AttributesIntoIterator<'a> {
    attributes: &'a Attributes,
    index: usize,
}

fn option_checker_string<'a>(attribute_type: &str,input:Option<&String>) -> Option<&'a str,AttributeValue<'a>> {
    let result = match input {
        Some(x) => Some((attribute_type,AttributeValue::String(x))),
        None => None,
    };
    result
}

fn option_checker_time<'a>(attribute_type: &str,input:Option<&DateTime<Utc>>) -> Option<&'a str,AttributeValue<'a>> {
    let result = match input {
        Some(x) => Some((attribute_type,AttributeValue::Time(x))),
        None => None,
    };
    result
}

impl<'a> Iterator for AttributesIntoIterator<'a> {
    type Item = (&'a str, AttributeValue<'a>);
    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.index {
            0 => Some(("id", AttributeValue::String(&self.attributes.id))),
            1 => Some(("ty", AttributeValue::String(&self.attributes.ty))),        
            2 => Some(("source", AttributeValue::String(&self.attributes.source))),
            3 => option_checker_string("datacontenttype",self.attributes.get_datacontenttype()), 
            4 => option_checker_string("dataschema",self.attributes.dataschema.get_dataschema()),
            5 => option_checker_string("subject",self.attributes.subject.get_subject()),
            6 => option_checker_time("time",self.attributes.time.get_time()),
            _ => return None,
        };
        self.index += 1;
        result
    }
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

    fn get_time(&self) -> Option<&DateTime<Utc>> {
        self.time.as_ref()
    }

    fn get_extension(&self, extension_name: &str) -> Option<&ExtensionValue> {
        self.extensions.get(extension_name)
    }

    fn iter_extensions(&self) -> std::collections::hash_map::Iter<String, ExtensionValue> {
        self.extensions.iter()
    }
}

impl AttributesWriter for Attributes {
    fn set_id(&mut self, id: impl Into<String>) {
        self.id = id.into()
    }

    fn set_source(&mut self, source: impl Into<String>) {
        self.source = source.into()
    }

    fn set_type(&mut self, ty: impl Into<String>) {
        self.ty = ty.into()
    }

    fn set_subject(&mut self, subject: Option<impl Into<String>>) {
        self.subject = subject.map(Into::into)
    }

    fn set_time(&mut self, time: Option<impl Into<DateTime<Utc>>>) {
        self.time = time.map(Into::into)
    }

    fn set_extension<'name, 'event: 'name>(
        &'event mut self,
        extension_name: &'name str,
        extension_value: impl Into<ExtensionValue>,
    ) {
        self.extensions
            .insert(extension_name.to_owned(), extension_value.into());
    }

    fn remove_extension<'name, 'event: 'name>(
        &'event mut self,
        extension_name: &'name str,
    ) -> Option<ExtensionValue> {
        self.extensions.remove(extension_name)
    }
}

impl DataAttributesWriter for Attributes {
    fn set_datacontenttype(&mut self, datacontenttype: Option<impl Into<String>>) {
        self.datacontenttype = datacontenttype.map(Into::into)
    }

    fn set_dataschema(&mut self, dataschema: Option<impl Into<String>>) {
        self.dataschema = dataschema.map(Into::into)
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
            extensions: HashMap::new(),
        }
    }
}