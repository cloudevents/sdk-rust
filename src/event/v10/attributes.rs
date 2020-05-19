use crate::event::attributes::{
    default_hostname, AttributeValue, AttributesConverter, DataAttributesWriter,
};
use crate::event::{AttributesReader, AttributesV03, AttributesWriter, SpecVersion};
use chrono::{DateTime, Utc};
use url::Url;
use uuid::Uuid;

pub(crate) const ATTRIBUTE_NAMES: [&'static str; 8] = [
    "specversion",
    "id",
    "type",
    "source",
    "datacontenttype",
    "dataschema",
    "subject",
    "time",
];

#[derive(PartialEq, Debug, Clone)]
pub struct Attributes {
    pub(crate) id: String,
    pub(crate) ty: String,
    pub(crate) source: Url,
    pub(crate) datacontenttype: Option<String>,
    pub(crate) dataschema: Option<Url>,
    pub(crate) subject: Option<String>,
    pub(crate) time: Option<DateTime<Utc>>,
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

pub struct AttributesIntoIterator<'a> {
    attributes: &'a Attributes,
    index: usize,
}

impl<'a> Iterator for AttributesIntoIterator<'a> {
    type Item = (&'a str, AttributeValue<'a>);
    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.index {
            0 => Some(("id", AttributeValue::String(&self.attributes.id))),
            1 => Some(("type", AttributeValue::String(&self.attributes.ty))),
            2 => Some(("source", AttributeValue::URIRef(&self.attributes.source))),
            3 => self
                .attributes
                .datacontenttype
                .as_ref()
                .map(|v| ("datacontenttype", AttributeValue::String(v))),
            4 => self
                .attributes
                .dataschema
                .as_ref()
                .map(|v| ("dataschema", AttributeValue::URI(v))),
            5 => self
                .attributes
                .subject
                .as_ref()
                .map(|v| ("subject", AttributeValue::String(v))),
            6 => self
                .attributes
                .time
                .as_ref()
                .map(|v| ("time", AttributeValue::Time(v))),
            _ => return None,
        };
        self.index += 1;
        if result.is_none() {
            return self.next();
        }
        result
    }
}

impl AttributesReader for Attributes {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn get_source(&self) -> &Url {
        &self.source
    }

    fn get_specversion(&self) -> SpecVersion {
        SpecVersion::V10
    }

    fn get_type(&self) -> &str {
        &self.ty
    }

    fn get_datacontenttype(&self) -> Option<&str> {
        self.datacontenttype.as_deref()
    }

    fn get_dataschema(&self) -> Option<&Url> {
        self.dataschema.as_ref()
    }

    fn get_subject(&self) -> Option<&str> {
        self.subject.as_deref()
    }

    fn get_time(&self) -> Option<&DateTime<Utc>> {
        self.time.as_ref()
    }
}

impl AttributesWriter for Attributes {
    fn set_id(&mut self, id: impl Into<String>) {
        self.id = id.into()
    }

    fn set_source(&mut self, source: impl Into<Url>) {
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

    fn set_dataschema(&mut self, dataschema: Option<impl Into<Url>>) {
        self.dataschema = dataschema.map(Into::into)
    }
}

impl Default for Attributes {
    fn default() -> Self {
        Attributes {
            id: Uuid::new_v4().to_string(),
            ty: "type".to_string(),
            source: default_hostname(),
            datacontenttype: None,
            dataschema: None,
            subject: None,
            time: None,
        }
    }
}

impl AttributesConverter for Attributes {
    fn into_v10(self) -> Self {
        self
    }

    fn into_v03(self) -> AttributesV03 {
        AttributesV03 {
            id: self.id,
            ty: self.ty,
            source: self.source,
            datacontenttype: self.datacontenttype,
            schemaurl: self.dataschema,
            subject: self.subject,
            time: self.time,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    #[test]
    fn iterator_test_v10() {
        let a = Attributes {
            id: String::from("1"),
            ty: String::from("someType"),
            source: Url::parse("https://example.net").unwrap(),
            datacontenttype: None,
            dataschema: None,
            subject: None,
            time: Some(DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp(61, 0),
                Utc,
            )),
        };
        let b = &mut a.into_iter();
        let time = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc);

        assert_eq!(("id", AttributeValue::String("1")), b.next().unwrap());
        assert_eq!(
            ("type", AttributeValue::String("someType")),
            b.next().unwrap()
        );
        assert_eq!(
            (
                "source",
                AttributeValue::URIRef(&Url::parse("https://example.net").unwrap())
            ),
            b.next().unwrap()
        );
        assert_eq!(("time", AttributeValue::Time(&time)), b.next().unwrap());
    }
}
