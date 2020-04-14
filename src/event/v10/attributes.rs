use crate::event::attributes::{DataAttributesWriter, AttributeValue};
use crate::event::{AttributesReader, AttributesWriter, ExtensionValue, SpecVersion};
use chrono::{DateTime, Utc};
use chrono::NaiveDate;
use hostname::get_hostname;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Attributes {
    id: String,
    #[serde(rename = "type")]
    ty: String,
    source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    datacontenttype: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dataschema: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    time: Option<DateTime<Utc>>,
    #[serde(flatten)]
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

fn option_to_time(input:&Option<DateTime<Utc>>) -> &DateTime<Utc> {
    let result = match *input {
        Some(x) => &x,
        None => &DateTime::<Utc>::from_utc(NaiveDate::from_ymd(1970, 1, 1).and_hms(0, 0, 0), Utc),
    };
    result
}

fn option_to_string(input:&Option<String>) -> &str {
    let result = match *input {
        Some(x) => &x[..],
        None => "",
    };
    result
}

struct AttributesIntoIterator<'a> {
    attributes: &'a Attributes,
    index: usize,
}

impl<'a> Iterator for AttributesIntoIterator<'a> {
    type Item = (&'a str, AttributeValue<'a>);
    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.index {
            0 => ("id", AttributeValue::String(&self.attributes.id)),
            1 => ("ty", AttributeValue::String(&self.attributes.ty)),
            2 => ("source", AttributeValue::String(&self.attributes.source)),
            3 => ("datacontenttype", AttributeValue::String(option_to_string(&self.attributes.datacontenttype))),
            4 => ("dataschema", AttributeValue::String(option_to_string(&self.attributes.dataschema))),
            5 => ("subject", AttributeValue::String(option_to_string(&self.attributes.subject))),
            6 => ("time", AttributeValue::Time(option_to_time(&self.attributes.time))),
            _ => return None,
        };
        self.index += 1;
        Some(result)
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
