use super::SpecVersion;
use crate::event::{AttributesV10, ExtensionValue};
use chrono::{DateTime, FixedOffset};

/// Trait to get [CloudEvents Context attributes](https://github.com/cloudevents/spec/blob/master/spec.md#context-attributes).
pub trait AttributesReader {
    /// Get the [id](https://github.com/cloudevents/spec/blob/master/spec.md#id).
    fn get_id(&self) -> &str;
    /// Get the [source](https://github.com/cloudevents/spec/blob/master/spec.md#source-1).
    fn get_source(&self) -> &str;
    /// Get the [specversion](https://github.com/cloudevents/spec/blob/master/spec.md#specversion).
    fn get_specversion(&self) -> SpecVersion;
    /// Get the [type](https://github.com/cloudevents/spec/blob/master/spec.md#type).
    fn get_type(&self) -> &str;
    /// Get the [datacontenttype](https://github.com/cloudevents/spec/blob/master/spec.md#datacontenttype).
    fn get_datacontenttype(&self) -> Option<&str>;
    /// Get the [dataschema](https://github.com/cloudevents/spec/blob/master/spec.md#dataschema).
    fn get_dataschema(&self) -> Option<&str>;
    /// Get the [subject](https://github.com/cloudevents/spec/blob/master/spec.md#subject).
    fn get_subject(&self) -> Option<&str>;
    /// Get the [time](https://github.com/cloudevents/spec/blob/master/spec.md#time).
    fn get_time(&self) -> Option<&DateTime<FixedOffset>>;
    /// Get the [extension](https://github.com/cloudevents/spec/blob/master/spec.md#extension-context-attributes) named `extension_name`
    fn get_extension(&self, extension_name: &str) -> Option<&ExtensionValue>;
    /// Get all the [extensions](https://github.com/cloudevents/spec/blob/master/spec.md#extension-context-attributes)
    fn get_extensions(&self) -> Vec<(&str, &ExtensionValue)>;
}

pub trait AttributesWriter {
    fn set_id(&mut self, id: impl Into<String>);
    fn set_source(&mut self, source: impl Into<String>);
    fn set_type(&mut self, ty: impl Into<String>);
    fn set_subject(&mut self, subject: Option<impl Into<String>>);
    fn set_time(&mut self, time: Option<impl Into<DateTime<FixedOffset>>>);
    fn set_extension<'name, 'event: 'name>(
        &'event mut self,
        extension_name: &'name str,
        extension_value: impl Into<ExtensionValue>,
    );
    fn remove_extension<'name, 'event: 'name>(
        &'event mut self,
        extension_name: &'name str,
    ) -> Option<ExtensionValue>;
}

pub(crate) trait DataAttributesWriter {
    fn set_datacontenttype(&mut self, datacontenttype: Option<impl Into<String>>);
    fn set_dataschema(&mut self, dataschema: Option<impl Into<String>>);
}

pub enum Attributes {
    V10(AttributesV10),
}

impl AttributesReader for Attributes {
    fn get_id(&self) -> &str {
        match self {
            Attributes::V10(a) => a.get_id(),
        }
    }

    fn get_source(&self) -> &str {
        match self {
            Attributes::V10(a) => a.get_source(),
        }
    }

    fn get_specversion(&self) -> SpecVersion {
        match self {
            Attributes::V10(a) => a.get_specversion(),
        }
    }

    fn get_type(&self) -> &str {
        match self {
            Attributes::V10(a) => a.get_type(),
        }
    }

    fn get_datacontenttype(&self) -> Option<&str> {
        match self {
            Attributes::V10(a) => a.get_datacontenttype(),
        }
    }

    fn get_dataschema(&self) -> Option<&str> {
        match self {
            Attributes::V10(a) => a.get_dataschema(),
        }
    }

    fn get_subject(&self) -> Option<&str> {
        match self {
            Attributes::V10(a) => a.get_subject(),
        }
    }

    fn get_time(&self) -> Option<&DateTime<FixedOffset>> {
        match self {
            Attributes::V10(a) => a.get_time(),
        }
    }

    fn get_extension(&self, extension_name: &str) -> Option<&ExtensionValue> {
        match self {
            Attributes::V10(a) => a.get_extension(extension_name),
        }
    }

    fn get_extensions(&self) -> Vec<(&str, &ExtensionValue)> {
        match self {
            Attributes::V10(a) => a.get_extensions(),
        }
    }
}

impl AttributesWriter for Attributes {
    fn set_id(&mut self, id: impl Into<String>) {
        match self {
            Attributes::V10(a) => a.set_id(id),
        }
    }

    fn set_source(&mut self, source: impl Into<String>) {
        match self {
            Attributes::V10(a) => a.set_source(source),
        }
    }

    fn set_type(&mut self, ty: impl Into<String>) {
        match self {
            Attributes::V10(a) => a.set_type(ty),
        }
    }

    fn set_subject(&mut self, subject: Option<impl Into<String>>) {
        match self {
            Attributes::V10(a) => a.set_subject(subject),
        }
    }

    fn set_time(&mut self, time: Option<impl Into<DateTime<FixedOffset>>>) {
        match self {
            Attributes::V10(a) => a.set_time(time),
        }
    }

    fn set_extension<'name, 'event: 'name>(
        &'event mut self,
        extension_name: &'name str,
        extension_value: impl Into<ExtensionValue>,
    ) {
        match self {
            Attributes::V10(a) => a.set_extension(extension_name, extension_value),
        }
    }

    fn remove_extension<'name, 'event: 'name>(
        &'event mut self,
        extension_name: &'name str,
    ) -> Option<ExtensionValue> {
        match self {
            Attributes::V10(a) => a.remove_extension(extension_name),
        }
    }
}

impl DataAttributesWriter for Attributes {
    fn set_datacontenttype(&mut self, datacontenttype: Option<impl Into<String>>) {
        match self {
            Attributes::V10(a) => a.set_datacontenttype(datacontenttype),
        }
    }

    fn set_dataschema(&mut self, dataschema: Option<impl Into<String>>) {
        match self {
            Attributes::V10(a) => a.set_dataschema(dataschema),
        }
    }
}
