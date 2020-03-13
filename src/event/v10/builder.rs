use crate::event::{Event, Data, Attributes, AttributesWriter, ExtensionValue};
use super::Attributes as AttributesV10;
use chrono::{Utc, DateTime};

pub struct EventBuilder {
    event: Event
}

impl EventBuilder {

    // This works as soon as we have an event version converter
    // pub fn from(event: Event) -> Self {
    //     EventBuilder { event }
    // }

    pub fn new() -> Self {
        EventBuilder {
            event: Event {
                attributes: Attributes::V10(AttributesV10::default()),
                data: None
            }
        }
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.event.set_id(id);
        return self
    }

    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.event.set_source(source);
        return self
    }

    pub fn ty(mut self, ty: impl Into<String>) -> Self {
        self.event.set_type(ty);
        return self
    }

    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.event.set_subject(Some(subject));
        return self
    }

    pub fn time(mut self, time: impl Into<DateTime<Utc>>) -> Self {
        self.event.set_time(Some(time));
        return self
    }

    pub fn extension(mut self, extension_name: &str, extension_value: impl Into<ExtensionValue>) -> Self {
        self.event.set_extension(extension_name, extension_value);
        return self
    }

    pub fn data(mut self, datacontenttype: impl Into<String>, data: impl Into<Data>) -> Self {
        self.event.write_data(datacontenttype, data);
        return self
    }

    pub fn data_with_schema(mut self, datacontenttype: impl Into<String>, dataschema: impl Into<String>, data: impl Into<Data>) -> Self {
        self.event.write_data_with_schema(datacontenttype, dataschema, data);
        return self
    }

    pub fn build(self) -> Event {
        self.event
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{AttributesReader, SpecVersion};

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
        let schema = "http://localhost:8080/schema";
        let data = serde_json::json!({
            "hello": "world"
        });

        let event = EventBuilder::new()
            .id(id)
            .source(source)
            .ty(ty)
            .subject(subject)
            .time(time)
            .extension(extension_name, extension_value)
            .data_with_schema(content_type, schema, data.clone())
            .build();

        assert_eq!(SpecVersion::V10, event.get_specversion());
        assert_eq!(id, event.get_id());
        assert_eq!(source, event.get_source());
        assert_eq!(ty, event.get_type());
        assert_eq!(subject, event.get_subject().unwrap());
        assert_eq!(time, event.get_time().unwrap().clone());
        assert_eq!(ExtensionValue::from(extension_value), event.get_extension(extension_name).unwrap().clone());
        assert_eq!(content_type, event.get_datacontenttype().unwrap());
        assert_eq!(schema, event.get_dataschema().unwrap());

        let event_data: serde_json::Value = event.try_get_data().unwrap().unwrap();
        assert_eq!(data, event_data);
    }

}
