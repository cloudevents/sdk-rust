use super::Attributes as AttributesV03;
use crate::event::{Attributes, Data, Event, ExtensionValue, EventBuilderError, TryIntoUrl, TryIntoTime};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use url::Url;

/// Builder to create a CloudEvent V0.3
#[derive(Clone)]
pub struct EventBuilder {
    id: Option<String>,
    ty: Option<String>,
    source: Option<Url>,
    datacontenttype: Option<String>,
    schemaurl: Option<Url>,
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
                EventBuilderError::ParseUrlError {attribute_name: "schemaurl", source: e}
            )
        };
        self.data = Some(data.into());
        self
    }
}

impl From<Event> for EventBuilder {
    fn from(event: Event) -> Self {
        let attributes = match event.attributes.into_v03() {
            Attributes::V03(attr) => attr,
            // This branch is unreachable because into_v03() returns
            // always a Attributes::V03
            _ => unreachable!()
        };

        EventBuilder {
            id: Some(attributes.id),
            ty: Some(attributes.ty),
            source: Some(attributes.source),
            datacontenttype: attributes.datacontenttype,
            schemaurl: attributes.schemaurl,
            subject: attributes.subject,
            time: attributes.time,
            data: event.data,
            extensions: event.extensions,
            error: None
        }
    }
}

impl Default for EventBuilder {
    fn default() -> Self {
        Self::from(Event::default())
    }
}

impl crate::event::builder::EventBuilder for EventBuilder {
    fn new() -> Self {
        EventBuilder {
            id: None,
            ty: None,
            source: None,
            datacontenttype: None,
            schemaurl: None,
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
                    attributes: Attributes::V03(AttributesV03 {
                        id: self.id.ok_or(EventBuilderError::MissingRequiredAttribute {attribute_name: "id"})?,
                        ty: self.ty.ok_or(EventBuilderError::MissingRequiredAttribute {attribute_name: "type"})?,
                        source: self.source.ok_or(EventBuilderError::MissingRequiredAttribute {attribute_name: "source"})?,
                        datacontenttype: self.datacontenttype,
                        schemaurl: self.schemaurl,
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
