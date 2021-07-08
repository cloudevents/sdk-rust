use super::Attributes as AttributesV10;
use crate::event::types::*;
use crate::event::{
    Attributes, Data, Event, EventBuilderError, ExtensionValue, TryIntoTime, TryIntoUri,
    UriReference,
};
use crate::message::MessageAttributeValue;
use std::collections::HashMap;
use std::convert::TryInto;

/// Builder to create a CloudEvent V1.0
#[derive(Clone, Debug)]
pub struct EventBuilder {
    id: Option<String>,
    ty: Option<String>,
    source: Option<UriReference>,
    datacontenttype: Option<String>,
    dataschema: Option<Uri>,
    subject: Option<String>,
    time: Option<DateTime<Utc>>,
    data: Option<Data>,
    extensions: HashMap<String, ExtensionValue>,
    error: Option<EventBuilderError>,
}

impl EventBuilder {
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn source(mut self, source: impl Into<String>) -> Self {
        let source = source.into();
        if source.is_empty() {
            self.error = Some(EventBuilderError::InvalidUriRefError {
                attribute_name: "source",
            });
        } else {
            self.source = Some(source);
        }
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
            Err(e) => {
                self.error = Some(EventBuilderError::ParseTimeError {
                    attribute_name: "time",
                    source: e,
                })
            }
        };
        self
    }

    pub fn extension(
        mut self,
        extension_name: &str,
        extension_value: impl Into<ExtensionValue>,
    ) -> Self {
        self.extensions
            .insert(extension_name.to_owned(), extension_value.into());
        self
    }

    pub(crate) fn data_without_content_type(mut self, data: impl Into<Data>) -> Self {
        self.data = Some(data.into());
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
        schemaurl: impl TryIntoUri,
        data: impl Into<Data>,
    ) -> Self {
        self.datacontenttype = Some(datacontenttype.into());
        match schemaurl.into_uri() {
            Ok(u) => self.dataschema = Some(u),
            Err(e) => {
                self.error = Some(EventBuilderError::ParseUriError {
                    attribute_name: "dataschema",
                    source: e,
                })
            }
        };
        self.data = Some(data.into());
        self
    }
}

impl From<Event> for EventBuilder {
    fn from(event: Event) -> Self {
        let attributes = match event.attributes.into_v10() {
            Attributes::V10(attr) => attr,
            // This branch is unreachable because into_v10() returns
            // always a Attributes::V10
            _ => unreachable!(),
        };

        EventBuilder {
            id: Some(attributes.id),
            ty: Some(attributes.ty),
            source: Some(attributes.source),
            datacontenttype: attributes.datacontenttype,
            dataschema: attributes.dataschema,
            subject: attributes.subject,
            time: attributes.time,
            data: event.data,
            extensions: event.extensions,
            error: None,
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
            dataschema: None,
            subject: None,
            time: None,
            data: None,
            extensions: Default::default(),
            error: None,
        }
    }

    fn build(self) -> Result<Event, EventBuilderError> {
        match self.error {
            Some(e) => Err(e),
            None => Ok(Event {
                attributes: Attributes::V10(AttributesV10 {
                    id: self.id.ok_or(EventBuilderError::MissingRequiredAttribute {
                        attribute_name: "id",
                    })?,
                    ty: self.ty.ok_or(EventBuilderError::MissingRequiredAttribute {
                        attribute_name: "type",
                    })?,
                    source: self
                        .source
                        .ok_or(EventBuilderError::MissingRequiredAttribute {
                            attribute_name: "source",
                        })?,
                    datacontenttype: self.datacontenttype,
                    dataschema: self.dataschema,
                    subject: self.subject,
                    time: self.time,
                }),
                data: self.data,
                extensions: self.extensions,
            }),
        }
    }
}

impl crate::event::message::AttributesSerializer for EventBuilder {
    fn serialize_attribute(
        &mut self,
        name: &str,
        value: MessageAttributeValue,
    ) -> crate::message::Result<()> {
        match name {
            "id" => self.id = Some(value.to_string()),
            "type" => self.ty = Some(value.to_string()),
            "source" => self.source = Some(value.to_string()),
            "datacontenttype" => self.datacontenttype = Some(value.to_string()),
            "dataschema" => self.dataschema = Some(value.try_into()?),
            "subject" => self.subject = Some(value.to_string()),
            "time" => self.time = Some(value.try_into()?),
            _ => {
                return Err(crate::message::Error::UnknownAttribute {
                    name: name.to_string(),
                })
            }
        }
        Ok(())
    }
}
