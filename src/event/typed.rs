use crate::event::{AttributesReader, AttributesWriter, Data, Event};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::convert::TryFrom;
use std::fmt;

/// Trait for types that can be associated with a CloudEvents event type.
///
/// This trait enables strongly-typed CloudEvents handling by associating
/// a Rust type with a specific CloudEvents `type` identifier. This allows
/// for validated conversions between generic `Event` and typed payloads.
///
/// # Example
///
/// ```ignore
/// use cloudevents::event::CloudEventPayload;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Clone)]
/// struct UserCreated {
///     user_id: String,
///     email: String,
/// }
///
/// impl CloudEventPayload for UserCreated {
///     const EVENT_TYPE: &'static str = "com.example.user.created";
/// }
/// ```
pub trait CloudEventPayload: Serialize + DeserializeOwned + Sized {
    /// The CloudEvents `type` attribute value associated with this payload type.
    const EVENT_TYPE: &'static str;
}

/// Error types for typed event operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedEventError {
    /// The CloudEvent's type does not match the expected type.
    TypeMismatch { expected: String, actual: String },
    /// The event data could not be deserialized into the target type.
    DeserializationError(String),
    /// The typed payload could not be serialized into event data.
    SerializationError(String),
    /// The event has no data, but data was expected.
    MissingData,
}

impl fmt::Display for TypedEventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypedEventError::TypeMismatch { expected, actual } => {
                write!(
                    f,
                    "CloudEvent type mismatch: expected '{}', got '{}'",
                    expected, actual
                )
            }
            TypedEventError::DeserializationError(msg) => {
                write!(f, "Failed to deserialize event data: {}", msg)
            }
            TypedEventError::SerializationError(msg) => {
                write!(f, "Failed to serialize event data: {}", msg)
            }
            TypedEventError::MissingData => {
                write!(f, "Event has no data but data was expected")
            }
        }
    }
}

impl std::error::Error for TypedEventError {}

/// A wrapper around `Event` that provides type-safe access to the event data.
///
/// `TypedEvent<T>` represents a CloudEvent with a type-safe payload. The payload
/// is authoritative and is kept separately from the event metadata. Converting
/// back to `Event` serializes the payload as JSON.
///
/// # Type Safety
///
/// The type parameter `T` must implement `CloudEventPayload`, which associates
/// it with a specific CloudEvents `type` value. When converting an `Event` to
/// `TypedEvent<T>`, the actual event type is validated against the expected type.
///
/// # Example
///
/// ```ignore
/// use cloudevents::{Event, TypedEvent, CloudEventPayload};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Clone)]
/// struct OrderCreated {
///     order_id: String,
///     amount: f64,
/// }
///
/// impl CloudEventPayload for OrderCreated {
///     const EVENT_TYPE: &'static str = "com.example.order.created";
/// }
///
/// async fn process_event(event: Event) -> Result<(), Box<dyn std::error::Error>> {
///     // Validates type matches and deserializes the payload
///     let typed: TypedEvent<OrderCreated> = event.try_into()?;
///
///     // Now we have full type safety
///     println!("Order ID: {}", typed.data().order_id);
///     println!("Amount: ${}", typed.data().amount);
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedEvent<T: CloudEventPayload> {
    event: Event,
    payload: T,
}

impl<T: CloudEventPayload> TypedEvent<T> {
    /// Creates a `TypedEvent` with the given event metadata and payload.
    ///
    /// The event type is set to `T::EVENT_TYPE`, and any existing event data is
    /// discarded so the typed payload remains authoritative. Use
    /// [`TryFrom<Event>`] to validate and convert an existing event.
    pub fn new(mut event: Event, payload: T) -> Self {
        event.set_type(T::EVENT_TYPE);
        event.data = None;
        event.set_datacontenttype(None as Option<String>);
        TypedEvent { event, payload }
    }

    /// Returns a reference to the typed payload.
    pub fn data(&self) -> &T {
        &self.payload
    }

    /// Consumes the `TypedEvent` and returns the typed payload.
    pub fn into_data(self) -> T {
        self.payload
    }

    /// Returns the event metadata without event data.
    pub fn event(&self) -> &Event {
        &self.event
    }

    /// Consumes the `TypedEvent` and serializes its payload into a CloudEvent.
    ///
    /// The payload is encoded as JSON and the event's data content type is set
    /// to `application/json`.
    pub fn try_into_event(mut self) -> Result<Event, TypedEventError> {
        let data = serde_json::to_value(&self.payload)
            .map_err(|e| TypedEventError::SerializationError(e.to_string()))?;
        self.event.set_data("application/json", data);
        Ok(self.event)
    }

    /// Consumes the `TypedEvent` and returns the event metadata and payload.
    pub fn into_parts(self) -> (Event, T) {
        (self.event, self.payload)
    }
}

/// Attempts to convert a generic `Event` into a `TypedEvent<T>`.
///
/// # Validation
///
/// This conversion performs two validations:
/// 1. The event's `type` attribute must match `T::EVENT_TYPE`
/// 2. The event data must deserialize successfully into type `T`
///
/// # Errors
///
/// Returns `Err` if:
/// - The event type doesn't match the expected type
/// - The event data cannot be deserialized into `T`
/// - The event has no data
impl<T: CloudEventPayload> TryFrom<Event> for TypedEvent<T> {
    type Error = TypedEventError;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        // Validate event type matches
        if event.ty() != T::EVENT_TYPE {
            return Err(TypedEventError::TypeMismatch {
                expected: T::EVENT_TYPE.to_string(),
                actual: event.ty().to_string(),
            });
        }

        // Extract and deserialize the data
        let payload = event
            .data()
            .ok_or(TypedEventError::MissingData)
            .and_then(|data| deserialize_payload::<T>(data))?;

        let mut event = event;
        event.data = None;
        event.set_datacontenttype(None as Option<String>);

        Ok(TypedEvent { event, payload })
    }
}

/// Helper function to deserialize event data into the target type.
fn deserialize_payload<T: DeserializeOwned>(data: &Data) -> Result<T, TypedEventError> {
    match data {
        Data::Json(value) => serde_json::from_value(value.clone())
            .map_err(|e| TypedEventError::DeserializationError(e.to_string())),
        Data::Binary(bytes) => serde_json::from_slice(bytes)
            .map_err(|e| TypedEventError::DeserializationError(e.to_string())),
        Data::String(s) => serde_json::from_str(s)
            .map_err(|e| TypedEventError::DeserializationError(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EventBuilder, EventBuilderV10};
    use serde::{Deserialize, Serialize};
    use std::convert::TryInto;
    use url::Url;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    struct TestPayload {
        user_id: String,
        email: String,
    }

    impl CloudEventPayload for TestPayload {
        const EVENT_TYPE: &'static str = "com.example.test.created";
    }

    #[test]
    fn test_typed_event_conversion_success() {
        let payload = TestPayload {
            user_id: "123".to_string(),
            email: "test@example.com".to_string(),
        };

        let event = EventBuilderV10::new()
            .id("test-id")
            .source(Url::parse("http://example.com").unwrap())
            .ty("com.example.test.created")
            .data("application/json", serde_json::to_value(&payload).unwrap())
            .build()
            .unwrap();

        let typed: TypedEvent<TestPayload> = event.try_into().unwrap();
        assert_eq!(typed.data(), &payload);
        assert!(typed.event().data().is_none());

        let event = typed.try_into_event().unwrap();
        let recovered: TypedEvent<TestPayload> = event.try_into().unwrap();
        assert_eq!(recovered.data(), &payload);
    }

    #[test]
    fn test_typed_event_type_mismatch() {
        let payload = TestPayload {
            user_id: "123".to_string(),
            email: "test@example.com".to_string(),
        };

        let event = EventBuilderV10::new()
            .id("test-id")
            .source(Url::parse("http://example.com").unwrap())
            .ty("com.example.wrong.type")
            .data("application/json", serde_json::to_value(&payload).unwrap())
            .build()
            .unwrap();

        let result: Result<TypedEvent<TestPayload>, _> = event.try_into();
        assert!(matches!(result, Err(TypedEventError::TypeMismatch { .. })));
    }

    #[test]
    fn test_typed_event_missing_data() {
        let event = EventBuilderV10::new()
            .id("test-id")
            .source(Url::parse("http://example.com").unwrap())
            .ty("com.example.test.created")
            .build()
            .unwrap();

        let result: Result<TypedEvent<TestPayload>, _> = event.try_into();
        assert_eq!(result, Err(TypedEventError::MissingData));
    }

    #[test]
    fn test_typed_event_deserialization_error() {
        let event = EventBuilderV10::new()
            .id("test-id")
            .source(Url::parse("http://example.com").unwrap())
            .ty("com.example.test.created")
            .data("application/json", serde_json::json!({"invalid": "data"}))
            .build()
            .unwrap();

        let result: Result<TypedEvent<TestPayload>, _> = event.try_into();
        assert!(matches!(
            result,
            Err(TypedEventError::DeserializationError(_))
        ));
    }

    #[test]
    fn test_typed_event_data_access() {
        let payload = TestPayload {
            user_id: "456".to_string(),
            email: "another@example.com".to_string(),
        };

        let event = EventBuilderV10::new()
            .id("test-id-2")
            .source(Url::parse("http://example.com").unwrap())
            .ty("com.example.test.created")
            .data("application/json", serde_json::to_value(&payload).unwrap())
            .build()
            .unwrap();

        let typed: TypedEvent<TestPayload> = event.try_into().unwrap();
        assert_eq!(typed.data().user_id, "456");
        assert_eq!(typed.data().email, "another@example.com");
    }

    #[test]
    fn test_typed_event_event_access() {
        let payload = TestPayload {
            user_id: "789".to_string(),
            email: "access@example.com".to_string(),
        };

        let event = EventBuilderV10::new()
            .id("test-id-3")
            .source(Url::parse("http://example.com").unwrap())
            .ty("com.example.test.created")
            .data("application/json", serde_json::to_value(&payload).unwrap())
            .build()
            .unwrap();

        let typed: TypedEvent<TestPayload> = event.try_into().unwrap();
        assert_eq!(typed.event().id(), "test-id-3");
    }

    #[test]
    fn test_typed_event_into_parts() {
        let payload = TestPayload {
            user_id: "999".to_string(),
            email: "parts@example.com".to_string(),
        };

        let event = EventBuilderV10::new()
            .id("test-id-4")
            .source(Url::parse("http://example.com").unwrap())
            .ty("com.example.test.created")
            .data("application/json", serde_json::to_value(&payload).unwrap())
            .build()
            .unwrap();

        let typed: TypedEvent<TestPayload> = event.try_into().unwrap();
        let (recovered_event, recovered_payload) = typed.into_parts();

        assert_eq!(recovered_event.id(), "test-id-4");
        assert!(recovered_event.data().is_none());
        assert_eq!(recovered_payload, payload);
    }

    #[test]
    fn test_typed_event_new_sets_type_and_payload_is_authoritative() {
        let event = EventBuilderV10::new()
            .id("test-id-5")
            .source(Url::parse("http://example.com").unwrap())
            .ty("com.example.wrong.type")
            .data("application/json", serde_json::json!({"stale": true}))
            .build()
            .unwrap();
        let payload = TestPayload {
            user_id: "123".to_string(),
            email: "test@example.com".to_string(),
        };

        let typed = TypedEvent::new(event, payload.clone());
        assert_eq!(typed.event().ty(), TestPayload::EVENT_TYPE);
        assert!(typed.event().data().is_none());

        let event = typed.try_into_event().unwrap();
        assert_eq!(event.ty(), TestPayload::EVENT_TYPE);
        assert_eq!(event.datacontenttype(), Some("application/json"));
        assert_eq!(
            event.data(),
            Some(&Data::Json(serde_json::to_value(payload).unwrap()))
        );
    }
}
