//! Implements AMQP 1.0 binding for CloudEvents

use std::convert::TryFrom;

use fe2o3_amqp_lib::types::messaging::{ApplicationProperties, Message, Body, Data as AmqpData, Properties};
use fe2o3_amqp_lib::types::primitives::{Value, Binary};

use crate::Event;
use crate::message::Error;

/// Type alias for an AMQP 1.0 message
/// 
/// The generic parameter can be anything that implements `Serialize` and `Deserialize` but is of
/// no importance because all CloudEvents are using the `Body::Data` as the body section type. For 
/// convenience, this type alias chose `Value` as the value of the generic parameter
pub type AmqpMessage = Message<Value>;

pub struct AmqpCloudEvent {
    properties: Properties,
    application_properties: ApplicationProperties,
    data: Binary,
}

impl From<AmqpCloudEvent> for AmqpMessage {
    fn from(event: AmqpCloudEvent) -> Self {
        Message::builder()
            .properties(event.properties)
            .application_properties(event.application_properties)
            .data(event.data)
            .build()
    }
}

impl TryFrom<AmqpMessage> for AmqpCloudEvent {
    type Error = Error;

    fn try_from(value: AmqpMessage) -> Result<Self, Self::Error> {
        let data = match value.body {
            Body::Data(AmqpData(data)) => data,
            _ => return Err(Error::WrongEncoding {  })
        };
        let properties = value.properties
            .ok_or(Error::WrongEncoding {  })?;
        let application_properties = value.application_properties
            .ok_or(Error::WrongEncoding {  })?;
        Ok(Self {
            properties,
            application_properties,
            data,
        })
    }
}


