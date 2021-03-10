use crate::event::attributes::{default_hostname, AttributeValue, AttributesConverter};
use crate::event::{AttributesReader, AttributesV10, AttributesWriter, SpecVersion, UriReference};
use crate::message::{BinarySerializer, MessageAttributeValue};
use chrono::{DateTime, Utc};
use url::Url;
use uuid::Uuid;

pub(crate) const ATTRIBUTE_NAMES: [&str; 8] = [
    "specversion",
    "id",
    "type",
    "source",
    "datacontenttype",
    "schemaurl",
    "subject",
    "time",
];

/// Data structure representing [CloudEvents V0.3 context attributes](https://github.com/cloudevents/spec/blob/v0.3/spec.md#context-attributes)
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Attributes {
    pub(crate) id: String,
    pub(crate) ty: String,
    pub(crate) source: UriReference,
    pub(crate) datacontenttype: Option<String>,
    pub(crate) schemaurl: Option<Url>,
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

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct AttributesIntoIterator<'a> {
    pub(crate) attributes: &'a Attributes,
    pub(crate) index: usize,
}

impl<'a> Iterator for AttributesIntoIterator<'a> {
    type Item = (&'a str, AttributeValue<'a>);
    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.index {
            0 => Some(("specversion", AttributeValue::SpecVersion(SpecVersion::V03))),
            1 => Some(("id", AttributeValue::String(&self.attributes.id))),
            2 => Some(("type", AttributeValue::String(&self.attributes.ty))),
            3 => Some(("source", AttributeValue::URIRef(&self.attributes.source))),
            4 => self
                .attributes
                .datacontenttype
                .as_ref()
                .map(|v| ("datacontenttype", AttributeValue::String(v))),
            5 => self
                .attributes
                .schemaurl
                .as_ref()
                .map(|v| ("schemaurl", AttributeValue::URI(v))),
            6 => self
                .attributes
                .subject
                .as_ref()
                .map(|v| ("subject", AttributeValue::String(v))),
            7 => self
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
    fn id(&self) -> &str {
        &self.id
    }

    fn source(&self) -> &UriReference {
        &self.source
    }

    fn specversion(&self) -> SpecVersion {
        SpecVersion::V03
    }

    fn ty(&self) -> &str {
        &self.ty
    }

    fn datacontenttype(&self) -> Option<&str> {
        self.datacontenttype.as_deref()
    }

    fn dataschema(&self) -> Option<&Url> {
        self.schemaurl.as_ref()
    }

    fn subject(&self) -> Option<&str> {
        self.subject.as_deref()
    }

    fn time(&self) -> Option<&DateTime<Utc>> {
        self.time.as_ref()
    }
}

impl AttributesWriter for Attributes {
    fn set_id(&mut self, id: impl Into<String>) -> String {
        std::mem::replace(&mut self.id, id.into())
    }

    fn set_source(&mut self, source: impl Into<UriReference>) -> UriReference {
        std::mem::replace(&mut self.source, source.into())
    }

    fn set_type(&mut self, ty: impl Into<String>) -> String {
        std::mem::replace(&mut self.ty, ty.into())
    }

    fn set_subject(&mut self, subject: Option<impl Into<String>>) -> Option<String> {
        std::mem::replace(&mut self.subject, subject.map(Into::into))
    }

    fn set_time(&mut self, time: Option<impl Into<DateTime<Utc>>>) -> Option<DateTime<Utc>> {
        std::mem::replace(&mut self.time, time.map(Into::into))
    }

    fn set_datacontenttype(
        &mut self,
        datacontenttype: Option<impl Into<String>>,
    ) -> Option<String> {
        std::mem::replace(&mut self.datacontenttype, datacontenttype.map(Into::into))
    }

    fn set_dataschema(&mut self, dataschema: Option<impl Into<Url>>) -> Option<Url> {
        std::mem::replace(&mut self.schemaurl, dataschema.map(Into::into))
    }
}

impl Default for Attributes {
    fn default() -> Self {
        Attributes {
            id: Uuid::new_v4().to_string(),
            ty: "type".to_string(),
            source: default_hostname().to_string(),
            datacontenttype: None,
            schemaurl: None,
            subject: None,
            time: Some(Utc::now()),
        }
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

impl crate::event::message::AttributesDeserializer for super::Attributes {
    fn deserialize_attributes<R: Sized, V: BinarySerializer<R>>(
        self,
        mut visitor: V,
    ) -> crate::message::Result<V> {
        visitor = visitor.set_attribute("id", MessageAttributeValue::String(self.id))?;
        visitor = visitor.set_attribute("type", MessageAttributeValue::String(self.ty))?;
        visitor = visitor.set_attribute("source", MessageAttributeValue::UriRef(self.source))?;
        if self.datacontenttype.is_some() {
            visitor = visitor.set_attribute(
                "datacontenttype",
                MessageAttributeValue::String(self.datacontenttype.unwrap()),
            )?;
        }
        if self.schemaurl.is_some() {
            visitor = visitor.set_attribute(
                "schemaurl",
                MessageAttributeValue::Uri(self.schemaurl.unwrap()),
            )?;
        }
        if self.subject.is_some() {
            visitor = visitor.set_attribute(
                "subject",
                MessageAttributeValue::String(self.subject.unwrap()),
            )?;
        }
        if self.time.is_some() {
            visitor = visitor
                .set_attribute("time", MessageAttributeValue::DateTime(self.time.unwrap()))?;
        }
        Ok(visitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    #[test]
    fn iterator_test_v03() {
        let a = Attributes {
            id: String::from("1"),
            ty: String::from("someType"),
            source: "https://example.net".into(),
            datacontenttype: None,
            schemaurl: None,
            subject: None,
            time: Some(DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp(61, 0),
                Utc,
            )),
        };
        let b = &mut a.into_iter();
        let time = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc);

        assert_eq!(
            ("specversion", AttributeValue::SpecVersion(SpecVersion::V03)),
            b.next().unwrap()
        );
        assert_eq!(("id", AttributeValue::String("1")), b.next().unwrap());
        assert_eq!(
            ("type", AttributeValue::String("someType")),
            b.next().unwrap()
        );
        assert_eq!(
            (
                "source",
                AttributeValue::URIRef(&"https://example.net".to_string())
            ),
            b.next().unwrap()
        );
        assert_eq!(("time", AttributeValue::Time(&time)), b.next().unwrap());
    }
}
