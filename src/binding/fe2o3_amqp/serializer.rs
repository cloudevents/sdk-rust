use fe2o3_amqp_types::messaging::{ApplicationProperties, Data as AmqpData};
use fe2o3_amqp_types::primitives::{Binary, SimpleValue, Symbol};

use crate::binding::header_prefix;
use crate::message::StructuredSerializer;
use crate::{
    event::SpecVersion,
    message::{BinarySerializer, Error, MessageAttributeValue},
};

use super::constants::DATACONTENTTYPE;
use super::{AmqpBody, EventMessage, ATTRIBUTE_PREFIX};

impl BinarySerializer<EventMessage> for EventMessage {
    fn set_spec_version(mut self, spec_version: SpecVersion) -> crate::message::Result<Self> {
        let key = String::from("cloudEvents:specversion");
        let value = String::from(spec_version.as_str());
        self.application_properties
            .get_or_insert(ApplicationProperties::default())
            .insert(key, SimpleValue::from(value));
        Ok(self)
    }

    fn set_attribute(
        mut self,
        name: &str,
        value: MessageAttributeValue,
    ) -> crate::message::Result<Self> {
        // For the binary mode, the AMQP content-type property field value maps directly to the
        // CloudEvents datacontenttype attribute.
        //
        // All CloudEvents attributes with exception of datacontenttype MUST be individually mapped
        // to and from the AMQP application-properties section.
        if name == DATACONTENTTYPE {
            self.content_type = match value {
                MessageAttributeValue::String(s) => Some(Symbol::from(s)),
                _ => return Err(Error::WrongEncoding {}),
            }
        } else {
            // CloudEvent attributes are prefixed with "cloudEvents:" for use in the
            // application-properties section
            let key = header_prefix(ATTRIBUTE_PREFIX, name);
            let value = SimpleValue::from(value);
            self.application_properties
                .get_or_insert(ApplicationProperties::default())
                .insert(key, value);
        }

        Ok(self)
    }

    // Extension attributes are always serialized according to binding rules like standard
    // attributes. However this specification does not prevent an extension from copying event
    // attribute values to other parts of a message, in order to interact with non-CloudEvents
    // systems that also process the message. Extension specifications that do this SHOULD specify
    // how receivers are to interpret messages if the copied values differ from the cloud-event
    // serialized values.
    fn set_extension(
        mut self,
        name: &str,
        value: MessageAttributeValue,
    ) -> crate::message::Result<Self> {
        let key = name.to_string();
        let value = SimpleValue::from(value);
        self.application_properties
            .get_or_insert(ApplicationProperties::default())
            .insert(key, value);
        Ok(self)
    }

    fn end_with_data(mut self, bytes: Vec<u8>) -> crate::message::Result<Self> {
        let data = Binary::from(bytes);
        self.body = AmqpBody::Data(AmqpData(data));
        Ok(self)
    }

    fn end(self) -> crate::message::Result<Self> {
        Ok(self)
    }
}

impl StructuredSerializer<EventMessage> for EventMessage {
    fn set_structured_event(mut self, bytes: Vec<u8>) -> crate::message::Result<Self> {
        self.content_type = Some(Symbol::from("application/cloudevents+json; charset=utf-8"));
        self.body = AmqpBody::Data(AmqpData(Binary::from(bytes)));
        Ok(self)
    }
}
