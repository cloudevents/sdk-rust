use crate::{
    message::{Result, StructuredDeserializer},
    Event,
};

use nats_lib as nats;

impl StructuredDeserializer for nats::Message {
    fn deserialize_structured<R: Sized, V: crate::message::StructuredSerializer<R>>(
        self,
        serializer: V,
    ) -> crate::message::Result<R> {
        serializer.set_structured_event(self.data.to_vec())
    }
}

/// Trait implemented by [`nats::Message`] to enable convenient deserialization to [`Event`]
///
/// Trait sealed <https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed>
pub trait MessageExt: private::Sealed {
    fn to_event(&self) -> Result<Event>;
}

impl MessageExt for nats::Message {
    fn to_event(&self) -> Result<Event> {
        StructuredDeserializer::into_event(self.to_owned())
    }
}

mod private {
    use nats_lib as nats;

    // Sealing the MessageExt
    pub trait Sealed {}
    impl Sealed for nats::Message {}
}

#[cfg(test)]
mod tests {
    use crate::test::fixtures;
    use nats_lib as nats;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_structured_deserialize_v10() {
        let expected = fixtures::v10::full_json_data_string_extension();

        let nats_message = nats::Message::new(
            "not_relevant",
            None,
            json!(expected).to_string().as_bytes(),
            None,
        );

        let actual = nats_message.to_event().unwrap();

        assert_eq!(expected, actual)
    }

    #[test]
    fn test_structured_deserialize_v03() {
        let expected = fixtures::v03::full_json_data();

        let nats_message = nats::Message::new(
            "not_relevant",
            None,
            json!(expected).to_string().as_bytes(),
            None,
        );

        let actual = nats_message.to_event().unwrap();

        assert_eq!(expected, actual)
    }
}
