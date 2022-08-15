//! Implements AMQP 1.0 binding for CloudEvents

use std::collections::HashMap;
use std::convert::TryFrom;

use fe2o3_amqp_lib::types::messaging::{
    ApplicationProperties, Body, Data as AmqpData, Message, Properties,
};
use fe2o3_amqp_lib::types::primitives::{Binary, SimpleValue, Symbol, Timestamp, Value};

use crate::event::{AttributeValue, Attributes, ExtensionValue};
use crate::message::{Error, MessageAttributeValue};
use crate::{Event, AttributesReader, Data};

use self::extensions::ExtensionsHandler;

pub mod extensions;

/// Type alias for an AMQP 1.0 message
///
/// The generic parameter can be anything that implements `Serialize` and `Deserialize` but is of
/// no importance because all CloudEvents are using the `Body::Data` as the body section type. For
/// convenience, this type alias chose `Value` as the value of the generic parameter
pub type AmqpMessage = Message<Value>;

pub type AmqpBody = Body<Value>;

pub type Extensions = HashMap<String, ExtensionValue>;

pub struct AmqpCloudEvent {
    content_type: Option<Symbol>,
    application_properties: ApplicationProperties,
    body: AmqpBody,
}

impl AmqpCloudEvent {
    pub fn with_extensions_handler<F>(handler: F) -> ExtensionsHandler<F>
    where
        F: FnOnce(Extensions) -> AmqpMessage
    {
        ExtensionsHandler::new(handler)
    }

    pub fn from_event(event: Event) -> Result<Self, Error> {
        Self::try_from(event)
    }
}

impl From<AmqpCloudEvent> for AmqpMessage {
    fn from(event: AmqpCloudEvent) -> Self {
        let mut properties = Properties::default();
        properties.content_type = event.content_type;
        Message {
            header: None,
            delivery_annotations: None,
            message_annotations: None,
            properties: Some(properties),
            application_properties: Some(event.application_properties),
            body: event.body,
            footer: None,
        }
    }
}

impl TryFrom<AmqpMessage> for AmqpCloudEvent {
    type Error = Error;

    fn try_from(value: AmqpMessage) -> Result<Self, Self::Error> {
        let body = match value.body {
            Body::Data(data) => Body::Data(data),
            _ => return Err(Error::WrongEncoding {}),
        };
        let content_type = value.properties.ok_or(Error::WrongEncoding {})?
            .content_type.take();
        let application_properties = value
            .application_properties
            .ok_or(Error::WrongEncoding {})?;
        Ok(Self {
            content_type,
            application_properties,
            body,
        })
    }
}

impl<'a> From<AttributeValue<'a>> for SimpleValue {
    fn from(value: AttributeValue) -> Self {
        match value {
            AttributeValue::SpecVersion(spec_ver) => {
                SimpleValue::String(String::from(spec_ver.as_str()))
            }
            AttributeValue::String(s) => SimpleValue::String(String::from(s)),
            AttributeValue::URI(uri) => SimpleValue::String(String::from(uri.as_str())),
            AttributeValue::URIRef(uri) => SimpleValue::String(uri.clone()),
            AttributeValue::Boolean(val) => SimpleValue::Bool(*val),
            AttributeValue::Integer(val) => SimpleValue::Long(*val),
            AttributeValue::Time(datetime) => {
                let millis = datetime.timestamp_millis();
                let timestamp = Timestamp::from_milliseconds(millis);
                SimpleValue::Timestamp(timestamp)
            }
        }
    }
}

impl<'a> From<AttributeValue<'a>> for Value {
    fn from(value: AttributeValue) -> Self {
        match value {
            AttributeValue::SpecVersion(spec_ver) => Value::String(String::from(spec_ver.as_str())),
            AttributeValue::String(s) => Value::String(String::from(s)),
            AttributeValue::URI(uri) => Value::String(String::from(uri.as_str())),
            AttributeValue::URIRef(uri) => Value::String(uri.clone()),
            AttributeValue::Boolean(val) => Value::Bool(*val),
            AttributeValue::Integer(val) => Value::Long(*val),
            AttributeValue::Time(datetime) => {
                let millis = datetime.timestamp_millis();
                let timestamp = Timestamp::from_milliseconds(millis);
                Value::Timestamp(timestamp)
            }
        }
    }
}

impl From<MessageAttributeValue> for SimpleValue {
    fn from(value: MessageAttributeValue) -> Self {
        match value {
            MessageAttributeValue::String(s) => SimpleValue::String(String::from(s)),
            MessageAttributeValue::Uri(uri) => SimpleValue::String(String::from(uri.as_str())),
            MessageAttributeValue::UriRef(uri) => SimpleValue::String(uri.clone()),
            MessageAttributeValue::Boolean(val) => SimpleValue::Bool(val),
            MessageAttributeValue::Integer(val) => SimpleValue::Long(val),
            MessageAttributeValue::DateTime(datetime) => {
                let millis = datetime.timestamp_millis();
                let timestamp = Timestamp::from_milliseconds(millis);
                SimpleValue::Timestamp(timestamp)
            }
            MessageAttributeValue::Binary(val) => SimpleValue::Binary(Binary::from(val)),
        }
    }
}

impl From<MessageAttributeValue> for Value {
    fn from(value: MessageAttributeValue) -> Self {
        match value {
            MessageAttributeValue::String(s) => Value::String(String::from(s)),
            MessageAttributeValue::Uri(uri) => Value::String(String::from(uri.as_str())),
            MessageAttributeValue::UriRef(uri) => Value::String(uri.clone()),
            MessageAttributeValue::Boolean(val) => Value::Bool(val),
            MessageAttributeValue::Integer(val) => Value::Long(val),
            MessageAttributeValue::DateTime(datetime) => {
                let millis = datetime.timestamp_millis();
                let timestamp = Timestamp::from_milliseconds(millis);
                Value::Timestamp(timestamp)
            }
            MessageAttributeValue::Binary(val) => Value::Binary(Binary::from(val)),
        }
    }
}

/// The `BinarySerializer`/`StructuredSerializer` traits are not implemented because 
/// "datacontenttype" needs special treatment in AMQP. However, `StructureSerializer` doesn't
/// provide access to "datacontenttype"
impl TryFrom<Event> for AmqpCloudEvent {
    type Error = Error;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        let (content_type, application_properties) = from_event_attributes(event.attributes);
        let body = from_event_data(event.data)?;

        Ok(Self {
            content_type,
            application_properties,
            body,
        })
    }
}

fn from_event_attributes(attributes: Attributes) -> (Option<Symbol>, ApplicationProperties) {
    let content_type = attributes.datacontenttype().map(Symbol::from);

    let mut application_properties = ApplicationProperties::default();
    for (key, value) in attributes.iter() {
        if key == "datacontenttype" {
            continue;
        } else {
            let key = format!("cloudEvents:{}", key);
            application_properties.insert(key, SimpleValue::from(value));
        }
    }
    (content_type, application_properties)
}

fn from_event_data(data: Option<Data>) -> Result<AmqpBody, Error> {
    let body = match data {
        Some(data) => match data {
            crate::Data::Binary(data) => Body::Data(AmqpData(Binary::from(data))),
            crate::Data::String(val) => Body::Data(AmqpData(Binary::from(val))),
            crate::Data::Json(val) => {
                let bytes = serde_json::to_vec(&val)?;
                Body::Data(AmqpData(Binary::from(bytes)))
            },
        },
        None => AmqpBody::Nothing,
    };
    Ok(body)
}

// impl BinarySerializer<AmqpCloudEvent> for AmqpCloudEvent {
//     fn set_spec_version(mut self, spec_version: SpecVersion) -> crate::message::Result<Self> {
//         let key = String::from("cloudEvents:specversion");
//         let value = String::from(spec_version.as_str());
//         self.application_properties.insert(key, SimpleValue::from(value));
//         Ok(self)
//     }

//     fn set_attribute(mut self, name: &str, value: MessageAttributeValue) -> crate::message::Result<Self> {
//         if name == "datacontenttype" {
//             self.properties.content_type = match value {
//                 MessageAttributeValue::String(s) => Some(Symbol::from(s)),
//                 _ => return Err(Error::WrongEncoding {  })
//             }
//         } else {
//             let key = format!("cloudEvents:{}", name);
//             let value = SimpleValue::from(value);
//             self.application_properties.insert(key, value);
//         }

//         Ok(self)
//     }

//     fn set_extension(self, name: &str, value: MessageAttributeValue) -> crate::message::Result<Self> {
//         todo!()
//     }

//     fn end_with_data(mut self, bytes: Vec<u8>) -> crate::message::Result<Self> {
//         let data = Binary::from(bytes);
//         self.body = Body::Data(AmqpData(data));
//         Ok(self)
//     }

//     fn end(self) -> crate::message::Result<Self> {
//         Ok(self)
//     }
// }

// impl StructuredSerializer<AmqpCloudEvent> for AmqpCloudEvent {
//     fn set_structured_event(self, bytes: Vec<u8>) -> crate::message::Result<Self> {
//         todo!()
//     }
// }
