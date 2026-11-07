# TypedEvent and CloudEventPayload Documentation

This document explains the `TypedEvent` and `CloudEventPayload` feature that enables type-safe, strongly-typed CloudEvents handling in Rust applications.

## Overview

`TypedEvent<T>` is an opt-in wrapper around the generic `Event` type that provides compile-time type safety for event data. It works in conjunction with the `CloudEventPayload` trait to establish a bidirectional association between Rust types and CloudEvents `type` identifiers.

### Design Philosophy

- **Non-breaking**: The core `Event` struct remains unchanged and unparameterized
- **Opt-in**: Applications choose when and where to use typed events
- **Interoperable**: Fully compatible with other CloudEvents SDKs and the CloudEvents specification
- **Ergonomic**: Reduces boilerplate and string-matching bugs

## CloudEventPayload Trait

The `CloudEventPayload` trait associates a Rust type with a CloudEvents `type` identifier.

### Definition

```rust
pub trait CloudEventPayload: Serialize + DeserializeOwned + Sized {
    const EVENT_TYPE: &'static str;
}
```

### Implementation

```rust
use cloudevents::CloudEventPayload;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
struct UserCreated {
    user_id: String,
    email: String,
}

impl CloudEventPayload for UserCreated {
    const EVENT_TYPE: &'static str = "com.example.users.created";
}
```

**Requirements:**
- The type must implement `Serialize` (for creating events from typed data)
- The type must implement `DeserializeOwned` (for validating incoming events)
- The `EVENT_TYPE` must be a valid CloudEvents type identifier

## TypedEvent Wrapper

`TypedEvent<T>` keeps the typed payload authoritative and stores event metadata
separately. Converting an existing `Event` validates its type and deserializes
its data. `TypedEvent::new` creates a typed event from metadata and payload,
setting the event type from `T::EVENT_TYPE`.

### Creating TypedEvent from Event

The primary way to create a `TypedEvent` is through the `TryFrom` conversion:

```rust
use cloudevents::{Event, TypedEvent};
use std::convert::TryInto;

let generic_event: Event = receive_event().await?;

// Validates type match and deserializes payload
let typed: TypedEvent<UserCreated> = generic_event.try_into()?;

// Fully typed access
println!("User ID: {}", typed.data().user_id);
```

### Accessing Data and Metadata

```rust
let typed: TypedEvent<UserCreated> = event.try_into()?;

// Access the typed payload
let payload: &UserCreated = typed.data();
println!("Email: {}", payload.email);

// Access event metadata. The typed payload is not duplicated in this Event.
let event: &Event = typed.event();
println!("Event ID: {}", event.id());

// Access CloudEvent metadata
let id: &str = typed.event().id();
let source: &UriReference = typed.event().source();
let time: Option<&DateTime<Utc>> = typed.event().time();
```

The payload is immutable through `TypedEvent`, so it cannot diverge from an
event-data copy. Convert it back to a generic event with the fallible
`try_into_event()` method, which serializes the payload as JSON:

```rust
let event: Event = typed.try_into_event()?;
```

Typed event producers can use `TypedEvent::new` without specifying the event
type string separately. The supplied `Event` is used for metadata; any existing
data is discarded, and the type is set to `T::EVENT_TYPE`.

```rust
let metadata = EventBuilderV10::new()
    .id("event-123")
    .source(Url::parse("https://example.com")?)
    .build()?;
let typed = TypedEvent::new(metadata, UserCreated {
    user_id: "user-123".into(),
    email: "alice@example.com".into(),
});
let event = typed.try_into_event()?;
```

### Decomposing TypedEvent

```rust
let (event, payload) = typed.into_parts();
// event contains metadata but no event data
// payload is now a UserCreated
```

## Error Handling

The `TypedEventError` enum provides detailed error information:

```rust
pub enum TypedEventError {
    /// Event type doesn't match expected type
    TypeMismatch { expected: String, actual: String },
    
    /// Event data cannot be deserialized into the target type
    DeserializationError(String),

    /// Typed payload cannot be serialized into event data
    SerializationError(String),
    
    /// Event has no data but data was expected
    MissingData,
}
```

### Example Error Handling

```rust
let result: Result<TypedEvent<UserCreated>, _> = event.try_into();

match result {
    Ok(typed) => println!("User ID: {}", typed.data().user_id),
    Err(TypedEventError::TypeMismatch { expected, actual }) => {
        eprintln!("Expected {} but got {}", expected, actual);
    }
    Err(TypedEventError::DeserializationError(e)) => {
        eprintln!("Failed to deserialize: {}", e);
    }
    Err(TypedEventError::SerializationError(e)) => {
        eprintln!("Failed to serialize: {}", e);
    }
    Err(TypedEventError::MissingData) => {
        eprintln!("Event has no data");
    }
}
```

## Practical Examples

### Single Event Type Processing

```rust
use cloudevents::{EventBuilderV10, CloudEventPayload, TypedEvent, EventBuilder};
use serde::{Serialize, Deserialize};
use url::Url;

#[derive(Serialize, Deserialize)]
struct OrderCreated {
    order_id: String,
    customer_id: String,
    amount: f64,
}

impl CloudEventPayload for OrderCreated {
    const EVENT_TYPE: &'static str = "com.example.orders.created";
}

async fn handle_order_created(event: Event) -> Result<()> {
    let typed: TypedEvent<OrderCreated> = event.try_into()?;
    
    // Process with full type safety
    println!("Processing order: {}", typed.data().order_id);
    
    // Access metadata
    println!("Event created at: {:?}", typed.event().time());
    
    Ok(())
}
```

### Multiple Event Types

Applications that process multiple event types can build a type-safe dispatcher:

```rust
enum DomainEvent {
    UserCreated(TypedEvent<UserCreated>),
    OrderPlaced(TypedEvent<OrderPlaced>),
    PaymentProcessed(TypedEvent<PaymentProcessed>),
}

fn parse_event(event: Event) -> Result<DomainEvent> {
    match event.ty() {
        UserCreated::EVENT_TYPE => {
            Ok(DomainEvent::UserCreated(event.try_into()?))
        }
        OrderPlaced::EVENT_TYPE => {
            Ok(DomainEvent::OrderPlaced(event.try_into()?))
        }
        PaymentProcessed::EVENT_TYPE => {
            Ok(DomainEvent::PaymentProcessed(event.try_into()?))
        }
        unknown => Err(format!("Unknown event type: {}", unknown).into()),
    }
}

async fn dispatch(domain_event: DomainEvent) -> Result<()> {
    match domain_event {
        DomainEvent::UserCreated(typed) => handle_user_created(typed).await?,
        DomainEvent::OrderPlaced(typed) => handle_order_placed(typed).await?,
        DomainEvent::PaymentProcessed(typed) => handle_payment_processed(typed).await?,
    }
    Ok(())
}
```

### Generic Event Composition

You can compose generic and typed event handling:

```rust
async fn process_generic_event(event: Event) -> Result<()> {
    // First, do generic processing
    println!("Received event: {}", event.id());
    
    // Then try to handle as a specific type
    match event.ty() {
        UserCreated::EVENT_TYPE => {
            let typed: TypedEvent<UserCreated> = event.try_into()?;
            handle_user_creation(typed).await?;
        }
        _ => {
            // Handle unknown types generically
            println!("Received unknown event type: {}", event.ty());
        }
    }
    
    Ok(())
}
```

## Backward Compatibility

This feature is fully backward compatible:

1. **No breaking changes to `Event`**: The core `Event` struct is unchanged
2. **Opt-in adoption**: Applications can use `TypedEvent` selectively
3. **Existing code unaffected**: All existing applications continue to work as-is
4. **Interoperability preserved**: CloudEvents remain fully compatible with other SDKs

## Comparison with Alternatives

### vs. Manual String Matching

**Before (without TypedEvent):**
```rust
if event.ty() == "com.example.users.created" {
    let data: UserCreated = serde_json::from_value(
        serde_json::to_value(event.data())?
    )?;
    println!("User: {}", data.email);
}
```

**After (with TypedEvent):**
```rust
if let Ok(typed) = TryInto::<TypedEvent<UserCreated>>::try_into(event) {
    println!("User: {}", typed.data().email);
}
```

### vs. Generic Event<T>

We chose `TypedEvent<T>` as a wrapper instead of making `Event<T>` generic because:

1. **Protocol bindings** work with untyped events (HTTP, Kafka, etc.)
2. **Dynamic event handling** is still supported
3. **Boundary handling** is cleaner with explicit conversion
4. **Interoperability** with other SDKs is maintained
5. **API stability** of the core `Event` type is preserved

## Best Practices

1. **Define domain types early**: Create your event types as part of your domain model
2. **Use consistent naming**: Follow a clear convention for `EVENT_TYPE` identifiers
3. **Leverage versioning**: Consider versioning your event types (e.g., `com.example.users.created.v1`)
4. **Handle errors explicitly**: Match on `TypedEventError` to provide meaningful error messages
5. **Document event schemas**: Use doc comments to document the structure of your payloads

## Limitations and Future Work

- **Optional data**: Currently requires event data to be present. Events without data will fail conversion.
- **Serialization**: Use `try_into_event()` to serialize the authoritative typed payload into JSON event data

## See Also

- [`CloudEventPayload`](https://docs.rs/cloudevents-sdk/latest/cloudevents/trait.CloudEventPayload.html)
- [`TypedEvent`](https://docs.rs/cloudevents-sdk/latest/cloudevents/struct.TypedEvent.html)
- [`TypedEventError`](https://docs.rs/cloudevents-sdk/latest/cloudevents/enum.TypedEventError.html)
- [CloudEvents Specification](https://cloudevents.io/)
- [Example: typed_events.rs](../examples/typed_events.rs)
