//! This module integrated the [cloudevents-sdk](https://docs.rs/cloudevents-sdk) with
//! [fe2o3-amqp](https://docs.rs/fe2o3-amqp/) to easily send and receive CloudEvents
//!
//! To send CloudEvents
//!
//! ```rust
//! use serde_json::json;
//! use fe2o3_amqp::{Connection, Sender, Session};
//! use cloudevents::{
//!     EventBuilder, EventBuilderV10, 
//!     binding::fe2o3_amqp::{EventMessage, AmqpMessage}
//! };
//!
//! // You need a running AMQP 1.0 broker to try out this example.
//! // With docker: docker run -it --rm -e ARTEMIS_USERNAME=guest -e ARTEMIS_PASSWORD=guest -p 5672:5672 vromero/activemq-artemis
//!
//! # async fn send_event() {
//!     let mut connection =
//!         Connection::open("cloudevents-sdk-rust", "amqp://guest:guest@localhost:5672")
//!             .await
//!             .unwrap();
//!     let mut session = Session::begin(&mut connection).await.unwrap();
//!     let mut sender = Sender::attach(&mut session, "sender", "q1").await.unwrap();
//!     
//!     let event = EventBuilderV10::new()
//!         .id("1")
//!         .ty("example.test")
//!         .source("localhost")
//!         .extension("ext-name", "AMQP")
//!         .data("application/json", json!({"hello": "world"}))
//!         .build()
//!         .unwrap();
//!     
//!     let event_message = EventMessage::from_binary_event(event).unwrap();
//!     let message = AmqpMessage::from(event_message);
//!     sender.send(message).await.unwrap()
//!         .accepted_or("not accepted").unwrap();
//!     
//!     sender.close().await.unwrap();
//!     session.end().await.unwrap();
//!     connection.close().await.unwrap();
//! # }
//! ```
//!
//! To receiver CloudEvents
//!
//! ```rust
//! use fe2o3_amqp::{Connection, Receiver, Session};
//! use cloudevents::{
//!     EventBuilderV10, message::MessageDeserializer,
//!     binding::fe2o3_amqp::{EventMessage, AmqpMessage}
//! };
//!
//! // You need a running AMQP 1.0 broker to try out this example.
//! // With docker: docker run -it --rm -e ARTEMIS_USERNAME=guest -e ARTEMIS_PASSWORD=guest -p 5672:5672 vromero/activemq-artemis
//!
//! # async fn receive_event() {
//!     let mut connection =
//!         Connection::open("cloudevents-sdk-rust", "amqp://guest:guest@localhost:5672")
//!             .await
//!             .unwrap();
//!     let mut session = Session::begin(&mut connection).await.unwrap();
//!     let mut receiver = Receiver::attach(&mut session, "receiver", "q1").await.unwrap();
//!     
//!     let delivery = receiver.recv().await.unwrap();
//!     receiver.accept(&delivery).await.unwrap();
//!     
//!     let message: AmqpMessage = delivery.into_message();
//!     let event_message = EventMessage::from(message);
//!     let event = MessageDeserializer::into_event(event_message).unwrap();
//!     
//!     receiver.close().await.unwrap();
//!     session.end().await.unwrap();
//!     connection.close().await.unwrap();
//! # }
//! ```

use std::convert::TryFrom;

use chrono::{TimeZone, Utc};
use fe2o3_amqp_types::messaging::{ApplicationProperties, Body, Message, Properties};
use fe2o3_amqp_types::primitives::{Binary, SimpleValue, Symbol, Timestamp, Value};

use crate::event::AttributeValue;
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
/// convenience, this type alias chooses `Value` as the value of the generic parameter
pub type AmqpMessage = Message<Value>;

/// Type alias for an AMQP 1.0 Body
///
/// The generic parameter can be anything that implements `Serialize` and `Deserialize` but is of
/// no importance because all CloudEvents are using the `Body::Data` as the body section type. For
/// convenience, this type alias chooses `Value` as the value of the generic parameter
pub type AmqpBody = Body<Value>;

/// This struct contains the necessary fields required for AMQP 1.0 binding.
/// It provides conversion between [`Event`] and [`AmqpMessage`]
///
/// # Examples
///
/// ## [`Event`] -> [`AmqpMessage`] in binary content mode
///
/// ```rust
/// use serde_json::json;
/// use fe2o3_amqp_types::messaging::Message;
/// use cloudevents::{EventBuilder, EventBuilderV10, binding::fe2o3_amqp::EventMessage};
/// 
/// let event = EventBuilderV10::new()
///     .id("1")
///     .ty("example.test")
///     .source("localhost")
///     .extension("ext-name", "AMQP")
///     .data("application/json", json!({"hello": "world"}))
///     .build()
///     .unwrap();
/// let event_message = EventMessage::from_binary_event(event).unwrap();
/// let amqp_message = Message::from(event_message);
/// ```
///
/// ## [`Event`] -> [`AmqpMessage`] in structured content mode
///
/// ```rust
/// use serde_json::json;
/// use fe2o3_amqp_types::messaging::Message;
/// use cloudevents::{EventBuilder, EventBuilderV10, binding::fe2o3_amqp::EventMessage};
/// 
/// let event = EventBuilderV10::new()
///     .id("1")
///     .ty("example.test")
///     .source("localhost")
///     .extension("ext-name", "AMQP")
///     .data("application/json", json!({"hello": "world"}))
///     .build()
///     .unwrap();
/// let event_message = EventMessage::from_structured_event(event).unwrap();
/// let amqp_message = Message::from(event_message);
/// ```
///
/// ## [`AmqpMessage`] -> [`Event`]
///
/// ```rust
/// use fe2o3_amqp::Receiver;
/// use cloudevents::{
///     message::MessageDeserializer,
///     binding::fe2o3_amqp::{AmqpMessage, EventMessage}
/// };
/// 
/// # async fn receive_event(receiver: &mut Receiver) {
///     let delivery = receiver.recv().await.unwrap();
///     receiver.accept(&delivery).await.unwrap();
///     let amqp_message: AmqpMessage = delivery.into_message();
///     let event_message = EventMessage::from(amqp_message);
///     let event = MessageDeserializer::into_event(event_message).unwrap();
/// # }
/// ```
pub struct EventMessage {
    pub content_type: Option<Symbol>,
    pub application_properties: Option<ApplicationProperties>,
    pub body: AmqpBody,
}

impl EventMessage {
    fn new() -> Self {
        Self {
            content_type: None,
            application_properties: None,
            body: Body::Nothing,
        }
    }

    /// Create an [`EventMessage`] from an event using a binary serializer
    pub fn from_binary_event(event: Event) -> Result<Self, Error> {
        BinaryDeserializer::deserialize_binary(event, Self::new())
    }

    /// Create an [`EventMessage`] from an event using a structured serializer
    pub fn from_structured_event(event: Event) -> Result<Self, Error> {
        StructuredDeserializer::deserialize_structured(event, Self::new())
    }
}

impl From<EventMessage> for AmqpMessage {
    fn from(event: EventMessage) -> Self {
        let properties = Properties {
            content_type: event.content_type,
            ..Default::default()
        };
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

impl From<AmqpMessage> for EventMessage {
    fn from(message: AmqpMessage) -> Self {
        let content_type = message.properties.and_then(|p| p.content_type);
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
            MessageAttributeValue::String(s) => SimpleValue::String(s),
            MessageAttributeValue::Uri(uri) => SimpleValue::String(String::from(uri.as_str())),
            MessageAttributeValue::UriRef(uri) => SimpleValue::String(uri),
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
            MessageAttributeValue::String(s) => Value::String(s),
            MessageAttributeValue::Uri(uri) => Value::String(String::from(uri.as_str())),
            MessageAttributeValue::UriRef(uri) => Value::String(uri),
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
