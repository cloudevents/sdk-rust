//! Implements AMQP 1.0 binding for CloudEvents

use std::collections::HashMap;
use std::convert::TryFrom;

use chrono::{TimeZone, Utc};
use fe2o3_amqp_lib::types::messaging::{ApplicationProperties, Body, Message, Properties};
use fe2o3_amqp_lib::types::primitives::{Binary, SimpleValue, Symbol, Timestamp, Value};

use crate::event::{AttributeValue, ExtensionValue};
use crate::message::{BinaryDeserializer, Error, MessageAttributeValue, StructuredDeserializer};
use crate::Event;

use self::constants::{
    prefixed, DATACONTENTTYPE, DATASCHEMA, ID, SOURCE, SPECVERSION, SUBJECT, TIME, TYPE,
};

const ATTRIBUTE_PREFIX: &str = "cloudEvents:";

pub mod deserializer;
pub mod serializer;

mod constants;

/// Type alias for an AMQP 1.0 message
///
/// The generic parameter can be anything that implements `Serialize` and `Deserialize` but is of
/// no importance because all CloudEvents are using the `Body::Data` as the body section type. For
/// convenience, this type alias chose `Value` as the value of the generic parameter
pub type AmqpMessage = Message<Value>;

pub type AmqpBody = Body<Value>;

pub type Extensions = HashMap<String, ExtensionValue>;

/// The receiver of the event can distinguish between the two modes by inspecting the content-type
/// message property field. If the value is prefixed with the CloudEvents media type
/// application/cloudevents, indicating the use of a known event format, the receiver uses
/// structured mode, otherwise it defaults to binary mode.
pub struct AmqpBinding {
    content_type: Option<Symbol>,
    application_properties: Option<ApplicationProperties>,
    body: AmqpBody,
}

impl AmqpBinding {
    fn new() -> Self {
        Self {
            content_type: None,
            application_properties: None,
            body: Body::Nothing,
        }
    }

    pub fn from_binary_event(event: Event) -> Result<Self, Error> {
        BinaryDeserializer::deserialize_binary(event, Self::new())
    }

    pub fn from_structured_event(event: Event) -> Result<Self, Error> {
        StructuredDeserializer::deserialize_structured(event, Self::new())
    }
}

impl From<AmqpBinding> for AmqpMessage {
    fn from(event: AmqpBinding) -> Self {
        let mut properties = Properties::default();
        properties.content_type = event.content_type;
        Message {
            header: None,
            delivery_annotations: None,
            message_annotations: None,
            properties: Some(properties),
            application_properties: event.application_properties,
            body: event.body,
            footer: None,
        }
    }
}

impl From<AmqpMessage> for AmqpBinding {
    fn from(message: AmqpMessage) -> Self {
        let content_type = message.properties.map(|p| p.content_type).flatten();
        Self {
            content_type,
            application_properties: message.application_properties,
            body: message.body,
        }
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

impl TryFrom<SimpleValue> for MessageAttributeValue {
    type Error = Error;

    fn try_from(value: SimpleValue) -> Result<Self, Self::Error> {
        match value {
            SimpleValue::Bool(val) => Ok(MessageAttributeValue::Boolean(val)),
            SimpleValue::Long(val) => Ok(MessageAttributeValue::Integer(val)),
            SimpleValue::Timestamp(val) => {
                let datetime = Utc.timestamp_millis(val.into_inner());
                Ok(MessageAttributeValue::DateTime(datetime))
            }
            SimpleValue::Binary(val) => Ok(MessageAttributeValue::Binary(val.into_vec())),
            SimpleValue::String(val) => Ok(MessageAttributeValue::String(val)),
            _ => Err(Error::WrongEncoding {}),
        }
    }
}

impl<'a> TryFrom<(&'a str, SimpleValue)> for MessageAttributeValue {
    type Error = Error;

    fn try_from((key, value): (&'a str, SimpleValue)) -> Result<Self, Self::Error> {
        match key {
            // String
            ID | prefixed::ID
            // String
            | SPECVERSION | prefixed::SPECVERSION
            // String
            | TYPE | prefixed::TYPE
            // String
            | DATACONTENTTYPE
            // String
            | SUBJECT | prefixed::SUBJECT => {
                let val = String::try_from(value).map_err(|_| Error::WrongEncoding {})?;
                Ok(MessageAttributeValue::String(val))
            },
            // URI-reference
            SOURCE | prefixed::SOURCE => {
                let val = String::try_from(value).map_err(|_| Error::WrongEncoding {})?;
                Ok(MessageAttributeValue::UriRef(val))
            },
            // URI
            DATASCHEMA | prefixed::DATASCHEMA => {
                let val = String::try_from(value).map_err(|_| Error::WrongEncoding {  })?;
                let url_val = url::Url::parse(&val)?;
                Ok(MessageAttributeValue::Uri(url_val))
            }
            // Timestamp
            TIME | prefixed::TIME => {
                let val = Timestamp::try_from(value).map_err(|_| Error::WrongEncoding {  })?;
                let datetime = Utc.timestamp_millis(val.into_inner());
                Ok(MessageAttributeValue::DateTime(datetime))
            }
            _ => {
                MessageAttributeValue::try_from(value)
            }
        }
    }
}
