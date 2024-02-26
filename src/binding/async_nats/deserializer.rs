use crate::{
    message::{Result, StructuredDeserializer},
    Event,
};

use async_nats_lib as async_nats;

impl StructuredDeserializer for async_nats::Message {
    fn deserialize_structured<R: Sized, V: crate::message::StructuredSerializer<R>>(
        self,
        serializer: V,
    ) -> crate::message::Result<R> {
        serializer.set_structured_event(self.payload.to_vec())
    }
}

/// Trait implemented by [`async_nats::Message`] to enable convenient deserialization to [`Event`]
///
/// Trait sealed <https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed>
pub trait MessageExt: private::Sealed {
    fn to_event(&self) -> Result<Event>;
}

impl MessageExt for async_nats::Message {
    fn to_event(&self) -> Result<Event> {
        let message = async_nats::Message {
            subject: self.subject.clone(),
            reply: self.reply.clone(),
            payload: self.payload.clone(),
            headers: self.headers.clone(),
            status: self.status.clone(),
            description: self.description.clone(),
            length: self.length,
        };
        StructuredDeserializer::into_event(message)
    }
}

mod private {
    use async_nats_lib as nats;

    // Sealing the MessageExt
    pub trait Sealed {}
    impl Sealed for nats::Message {}
}

#[cfg(test)]
mod tests {
    use crate::test::fixtures;
    use async_nats_lib as async_nats;
    use bytes::Bytes;
    use serde_json::json;
    use MessageExt;

    use super::*;

    #[test]
    fn test_structured_deserialize_v10() {
        let expected = fixtures::v10::full_json_data_string_extension();

        let nats_message = async_nats::Message {
            subject: "not_relevant".to_string(),
            reply: None,
            payload: Bytes::from(json!(expected).to_string()),
            headers: None,
            status: None,
            description: None,
            length: 0,
        };

        let actual = nats_message.to_event().unwrap();

        assert_eq!(expected, actual)
    }

    #[test]
    fn test_structured_deserialize_v03() {
        let expected = fixtures::v03::full_json_data();

        let nats_message = async_nats::Message {
            subject: "not_relevant".to_string(),
            reply: None,
            payload: Bytes::from(json!(expected).to_string()),
            headers: None,
            status: None,
            description: None,
            length: 0,
        };

        let actual = nats_message.to_event().unwrap();

        assert_eq!(expected, actual)
    }
}
