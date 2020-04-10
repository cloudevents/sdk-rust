use crate::event::attributes::{AttributesConverter, DataAttributesWriter};
use crate::event::AttributesV10;
use crate::event::{AttributesReader, AttributesWriter, SpecVersion};
use chrono::{DateTime, Utc};
use hostname::get_hostname;
use uuid::Uuid;

attributes_def!(
    Attributes,
    ATTRIBUTES_NAMES,
    {
        id: String {
            default: Uuid::new_v4().to_string(),
        },
        ty as "type": String {
            default: "rust.generated".to_string(),
        },
        source: String {
            default: get_hostname().unwrap_or("http://localhost/".to_string()),
        },
        datacontenttype: Option<String>,
        schemaurl: Option<String>,
        subject: Option<String>,
        time: Option<DateTime<Utc>>,
    }
);

impl AttributesReader for Attributes {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn get_source(&self) -> &str {
        &self.source
    }

    fn get_specversion(&self) -> SpecVersion {
        SpecVersion::V03
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
        match self.schemaurl.as_ref() {
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
}

impl DataAttributesWriter for Attributes {
    fn set_datacontenttype(&mut self, datacontenttype: Option<impl Into<String>>) {
        self.datacontenttype = datacontenttype.map(Into::into)
    }

    fn set_dataschema(&mut self, dataschema: Option<impl Into<String>>) {
        self.schemaurl = dataschema.map(Into::into)
    }
}

impl AttributesConverter for Attributes {
    fn into_v03(self) -> Self {
        self
    }

    fn into_v10(self) -> AttributesV10 {
        AttributesV10 {
            id: self.id,
            ty: self.ty,
            source: self.source,
            datacontenttype: self.datacontenttype,
            dataschema: self.schemaurl,
            subject: self.subject,
            time: self.time,
        }
    }
}
