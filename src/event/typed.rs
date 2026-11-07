use crate::event::{AttributesReader, Data, Event};
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
            TypedEventError::MissingData => {
                write!(f, "Event has no data but data was expected")
            }
        }
    }
}

impl std::error::Error for TypedEventError {}

/// A wrapper around `Event` that provides type-safe access to the event data.
///
/// `TypedEvent<T>` represents a CloudEvent with validated event type and
/// deserialized payload. It preserves all CloudEvent metadata while providing
/// compile-time safety for the event data.
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
    /// Creates a new `TypedEvent` with the given event and payload.
    ///
    /// # Arguments
    ///
    /// * `event` - The CloudEvent with metadata
    /// * `payload` - The strongly-typed event data
    ///
    /// # Note
    ///
    /// This function does not validate that the event's type matches `T::EVENT_TYPE`.
    /// Use `try_from` for validated conversion.
    pub fn new(event: Event, payload: T) -> Self {
        TypedEvent { event, payload }
    }

    /// Returns a reference to the typed payload.
    pub fn data(&self) -> &T {
        &self.payload
    }

    /// Returns a mutable reference to the typed payload.
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.payload
    }

    /// Consumes the `TypedEvent` and returns the typed payload.
    pub fn into_data(self) -> T {
        self.payload
    }

    /// Returns a reference to the underlying CloudEvent.
    pub fn event(&self) -> &Event {
        &self.event
    }

    /// Returns a mutable reference to the underlying CloudEvent.
    pub fn event_mut(&mut self) -> &mut Event {
        &mut self.event
    }

    /// Consumes the `TypedEvent` and returns the underlying CloudEvent and payload.
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
        assert_eq!(recovered_payload, payload);
    }
}
