use super::Attributes as AttributesV03;
use crate::event::{
    Attributes, Data, Event, EventBuilderError, ExtensionValue, TryIntoTime, TryIntoUrl,
    UriReference,
};
use crate::message::MessageAttributeValue;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::convert::TryInto;
use url::Url;

/// Builder to create a CloudEvent V0.3
#[derive(Clone, Debug)]
pub struct EventBuilder {
    id: Option<String>,
    ty: Option<String>,
    source: Option<UriReference>,
    datacontenttype: Option<String>,
    schemaurl: Option<Url>,
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
        schemaurl: impl TryIntoUrl,
        data: impl Into<Data>,
    ) -> Self {
        self.datacontenttype = Some(datacontenttype.into());
        match schemaurl.into_url() {
            Ok(u) => self.schemaurl = Some(u),
            Err(e) => {
                self.error = Some(EventBuilderError::ParseUrlError {
                    attribute_name: "schemaurl",
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
        let attributes = match event.attributes.into_v03() {
            Attributes::V03(attr) => attr,
            // This branch is unreachable because into_v03() returns
            // always a Attributes::V03
            _ => unreachable!(),
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
            schemaurl: None,
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
                attributes: Attributes::V03(AttributesV03 {
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
                    schemaurl: self.schemaurl,
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
            "schemaurl" => self.schemaurl = Some(value.try_into()?),
            "subject" => self.subject = Some(value.to_string()),
            "time" => self.time = Some(value.try_into()?),
            _ => {
                return Err(crate::message::Error::UnknownAttribute {
                    name: name.to_string(),
                })
            }
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::assert_match_pattern;
    use chrono::{DateTime, Utc};

    use crate::event::{
        AttributesReader, EventBuilder, EventBuilderError, ExtensionValue, SpecVersion,
    };
    use crate::EventBuilderV03;
    use std::convert::TryInto;
    use url::Url;

    #[test]
    fn build_event() {
        let id = "aaa";
        let source = "http://localhost:8080";
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

        let mut event = EventBuilderV03::new()
            .id(id)
            .source(source.clone())
            .ty(ty)
            .subject(subject)
            .time(time)
            .extension(extension_name, extension_value)
            .data_with_schema(content_type, schema.clone(), data.clone())
            .build()
            .unwrap();

        assert_eq!(SpecVersion::V03, event.specversion());
        assert_eq!(id, event.id());
        assert_eq!(source, event.source().clone());
        assert_eq!(ty, event.ty());
        assert_eq!(subject, event.subject().unwrap());
        assert_eq!(time, event.time().unwrap().clone());
        assert_eq!(
            ExtensionValue::from(extension_value),
            event.extension(extension_name).unwrap().clone()
        );
        assert_eq!(content_type, event.datacontenttype().unwrap());
        assert_eq!(schema, event.dataschema().unwrap().clone());

        let event_data: serde_json::Value = event.take_data().2.unwrap().try_into().unwrap();
        assert_eq!(data, event_data);
    }

    #[test]
    fn source_valid_relative_url() {
        let res = EventBuilderV03::new()
            .id("id1")
            .source("/source") // relative URL
            .ty("type")
            .build();
        assert_match_pattern!(res, Ok(_));
    }

    #[test]
    fn build_missing_id() {
        let res = EventBuilderV03::new()
            .source("http://localhost:8080")
            .build();
        assert_match_pattern!(
            res,
            Err(EventBuilderError::MissingRequiredAttribute {
                attribute_name: "id"
            })
        );
    }

    #[test]
    fn source_invalid_url() {
        let res = EventBuilderV03::new().source("").build();
        assert_match_pattern!(
            res,
            Err(EventBuilderError::InvalidUriRefError {
                attribute_name: "source",
            })
        );
    }

    #[test]
    fn default_builds() {
        let res = EventBuilderV03::default().build();
        assert_match_pattern!(res, Ok(_));
    }
}
