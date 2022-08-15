use std::collections::HashMap;

use fe2o3_amqp_types::messaging::{ApplicationProperties, Properties};

use crate::{event::ExtensionValue, message::Error, Event};

use super::{
    from_event_attributes, from_event_data, AmqpBody, AmqpCloudEvent, AmqpMessage, Extensions,
};

pub struct ExtensionsHandler<F>
where
    F: FnOnce(Extensions) -> AmqpMessage,
{
    handler: F,
}

impl<F> ExtensionsHandler<F>
where
    F: FnOnce(Extensions) -> AmqpMessage,
{
    pub fn new(handler: F) -> Self {
        Self { handler }
    }

    pub fn from_event(self, event: Event) -> Result<AmqpCloudEventExt<F>, Error> {
        let (content_type, application_properties) = from_event_attributes(event.attributes);
        let body = from_event_data(event.data)?;
        let inner = AmqpCloudEvent {
            content_type,
            application_properties,
            body,
        };
        Ok(AmqpCloudEventExt {
            inner,
            extensions: event.extensions,
            handler: self.handler,
        })
    }
}

pub struct AmqpCloudEventExt<F>
where
    F: FnOnce(Extensions) -> AmqpMessage,
{
    inner: AmqpCloudEvent,
    extensions: Extensions,
    handler: F,
}

impl<F> AmqpCloudEventExt<F> where F: FnOnce(Extensions) -> AmqpMessage {}

impl<F> From<AmqpCloudEventExt<F>> for AmqpMessage
where
    F: FnOnce(Extensions) -> AmqpMessage,
{
    fn from(mut value: AmqpCloudEventExt<F>) -> Self {
        let mut message = (value.handler)(value.extensions);

        // Set content_type to "datacontenttype"
        let properties = message.properties.get_or_insert(Properties::default());
        properties.content_type = value.inner.content_type;

        // Append ApplicationProperties
        let application_properties = message
            .application_properties
            .get_or_insert(ApplicationProperties::default());
        application_properties
            .0
            .append(&mut value.inner.application_properties.0);

        // Overrite the message body
        message.body = value.inner.body;

        message
    }
}
