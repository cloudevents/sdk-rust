use super::Attributes as AttributesV10;
use crate::event::{Attributes, AttributesWriter, Data, Event, ExtensionValue, EventBuilderError, TryIntoUrl, TryIntoTime};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use url::Url;

/// Builder to create a CloudEvent V1.0
#[derive(Clone)]
pub struct EventBuilder {
    id: Option<String>,
    ty: Option<String>,
    source: Option<Url>,
    datacontenttype: Option<String>,
    dataschema: Option<Url>,
    subject: Option<String>,
    time: Option<DateTime<Utc>>,
    data: Option<Data>,
    extensions: HashMap<String, ExtensionValue>,
    error: Option<EventBuilderError>
}

impl EventBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn source(mut self, source: impl TryIntoUrl) -> Self {
        match source.into_url() {
            Ok(u) => self.source = Some(u),
            Err(e) => self.error = Some(
                EventBuilderError::ParseUrlError {attribute_name: "source", source: e}
            )
        };
        self
    }

    pub fn ty(mut self, ty: impl Into<String>) -> Self {
        self.ty = Some(ty.into());
        self
    }

    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    pub fn time(mut self, time: impl TryIntoTime) -> Self {
        match time.into_time() {
            Ok(u) => self.time = Some(u),
            Err(e) => self.error = Some(
                EventBuilderError::ParseTimeError {attribute_name: "time", source: e}
            )
        };
        self
    }

    pub fn extension(
        mut self,
        extension_name: &str,
        extension_value: impl Into<ExtensionValue>,
    ) -> Self {
        self.extensions.insert(extension_name.to_owned(), extension_value.into());
        self
    }

    pub fn data(mut self, datacontenttype: impl Into<String>, data: impl Into<Data>) -> Self {
        self.datacontenttype = Some(datacontenttype.into());
        self.data = Some(data.into());
        self
    }

    pub fn data_with_schema(
        mut self,
        datacontenttype: impl Into<String>,
        schemaurl: impl TryIntoUrl,
        data: impl Into<Data>,
    ) -> Self {
        self.datacontenttype = Some(datacontenttype.into());
        match schemaurl.into_url() {
            Ok(u) => self.schemaurl = Some(u),
            Err(e) => self.error = Some(
                EventBuilderError::ParseUrlError {attribute_name: "dataschema", source: e}
            )
        };
        self.data = Some(data.into());
        self
    }
}

impl crate::event::builder::EventBuilder for EventBuilder {
    fn from(event: Event) -> Self {
        let attributes = match event.attributes.into_v10() {
            Attributes::V10(attr) => attr,
            // This branch is unreachable because into_v10() returns
            // always a Attributes::V10
            _ => unreachable!()
        };

        EventBuilder {
            id: Some(attributes.id),
            ty: Some(attributes.ty),
            source: Some(attributes.source),
            datacontenttype: attributes.datacontenttype,
            dataschema: attributes.schemaurl,
            subject: attributes.subject,
            time: attributes.time,
            data: event.data,
            extensions: event.extensions,
            error: None
        }
    }

    fn new() -> Self {
        EventBuilder {
            id: None,
            ty: None,
            source: None,
            datacontenttype: None,
            dataschema: None,
            subject: None,
            time: None,
            data: None,
            extensions: Default::default(),
            error: None
        }
    }

    fn build(self) -> Result<Event, EventBuilderError> {
        match self.error {
            Some(e) => Err(e),
            None => {
                Ok(Event{
                    attributes: Attributes::V10(AttributesV10 {
                        id: self.id.ok_or(EventBuilderError::MissingRequiredAttribute {attribute_name: "id"})?,
                        ty: self.ty.ok_or(EventBuilderError::MissingRequiredAttribute {attribute_name: "type"})?,
                        source: self.source.ok_or(EventBuilderError::MissingRequiredAttribute {attribute_name: "source"})?,
                        datacontenttype: self.datacontenttype,
                        dataschema: self.schemaurl,
                        subject: self.subject,
                        time: self.time
                    }),
                    data: self.data,
                    extensions: self.extensions
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::event::{AttributesReader, SpecVersion, EventBuilder, ExtensionValue};
    use url::Url;
    use chrono::{DateTime, Utc};

    #[test]
    fn build_event() {
        let id = "aaa";
        let source = Url::parse("http://localhost:8080").unwrap();
        let ty = "bbb";
        let subject = "francesco";
        let time: DateTime<Utc> = Utc::now();
        let extension_name = "ext";
        let extension_value = 10i64;
        let content_type = "application/json";
        let schema = Url::parse("http://localhost:8080/schema").unwrap();
        let data = serde_json::json!({
            "hello": "world"
        });

        let event = super::EventBuilder::new()
            .id(id)
            .source(source.clone())
            .ty(ty)
            .subject(subject)
            .time(time)
            .extension(extension_name, extension_value)
            .data_with_schema(content_type, schema.clone(), data.clone())
            .build()
            .unwrap();

        assert_eq!(SpecVersion::V10, event.get_specversion());
        assert_eq!(id, event.get_id());
        assert_eq!(source, event.get_source().clone());
        assert_eq!(ty, event.get_type());
        assert_eq!(subject, event.get_subject().unwrap());
        assert_eq!(time, event.get_time().unwrap().clone());
        assert_eq!(
            ExtensionValue::from(extension_value),
            event.get_extension(extension_name).unwrap().clone()
        );
        assert_eq!(content_type, event.get_datacontenttype().unwrap());
        assert_eq!(schema, event.get_dataschema().unwrap().clone());

        let event_data: serde_json::Value = event.try_get_data().unwrap().unwrap();
        assert_eq!(data, event_data);
    }
}
