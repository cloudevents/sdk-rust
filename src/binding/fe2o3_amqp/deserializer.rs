use std::convert::TryFrom;

use fe2o3_amqp_types::primitives::{SimpleValue, Symbol};

use crate::{
    binding::CLOUDEVENTS_JSON_HEADER,
    event::SpecVersion,
    message::{
        BinaryDeserializer, BinarySerializer, Encoding, Error, MessageAttributeValue,
        MessageDeserializer, Result, StructuredDeserializer, StructuredSerializer,
    },
};

use super::{
    constants::{prefixed, DATACONTENTTYPE},
    EventMessage, ATTRIBUTE_PREFIX,
};

impl BinaryDeserializer for EventMessage {
    fn deserialize_binary<R: Sized, V: BinarySerializer<R>>(
        mut self,
        mut serializer: V,
    ) -> Result<R> {
        use fe2o3_amqp_types::messaging::Body;

        // specversion
        let spec_version = {
            let value = self
                .application_properties
                .as_mut()
                .ok_or(Error::WrongEncoding {})?
                .remove(prefixed::SPECVERSION)
                .ok_or(Error::WrongEncoding {})
                .map(|val| match val {
                    SimpleValue::String(s) => Ok(s),
                    _ => Err(Error::WrongEncoding {}),
                })??;
            SpecVersion::try_from(&value[..])?
        };
        serializer = serializer.set_spec_version(spec_version.clone())?;

        // datacontenttype
        serializer = match self.content_type {
            Some(Symbol(content_type)) => serializer
                .set_attribute(DATACONTENTTYPE, MessageAttributeValue::String(content_type))?,
            None => serializer,
        };

        // remaining attributes
        let attributes = spec_version.attribute_names();

        if let Some(application_properties) = self.application_properties {
            for (key, value) in application_properties.0.into_iter() {
                if let Some(key) = key.strip_prefix(ATTRIBUTE_PREFIX) {
                    if attributes.contains(&key) {
                        let value = MessageAttributeValue::try_from((key, value))?;
                        serializer = serializer.set_attribute(key, value)?;
                    } else {
                        let value = MessageAttributeValue::try_from(value)?;
                        serializer = serializer.set_extension(key, value)?;
                    }
                }
            }
        }

        match self.body {
            Body::Data(data) => {
                let bytes = data.0.into_vec();
                serializer.end_with_data(bytes)
            }
            Body::Nothing => serializer.end(),
            Body::Sequence(_) | Body::Value(_) => Err(Error::WrongEncoding {}),
        }
    }
}

impl StructuredDeserializer for EventMessage {
    fn deserialize_structured<R: Sized, V: StructuredSerializer<R>>(
        self,
        serializer: V,
    ) -> Result<R> {
        use fe2o3_amqp_types::messaging::Body;
        let bytes = match self.body {
            Body::Data(data) => data.0.into_vec(),
            Body::Nothing => vec![],
            Body::Sequence(_) | Body::Value(_) => return Err(Error::WrongEncoding {}),
        };
        serializer.set_structured_event(bytes)
    }
}

impl MessageDeserializer for EventMessage {
    fn encoding(&self) -> Encoding {
        match self
            .content_type
            .as_ref()
            .map(|s| s.starts_with(CLOUDEVENTS_JSON_HEADER))
        {
            Some(true) => Encoding::STRUCTURED,
            Some(false) => Encoding::BINARY,
            None => Encoding::UNKNOWN,
        }
    }
}
