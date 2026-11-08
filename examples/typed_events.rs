// Example: Type-safe CloudEvents handling with TypedEvent
//
// This example demonstrates how to use TypedEvent and CloudEventPayload
// to create strongly-typed, compile-time-safe CloudEvents handling.

use cloudevents::{AttributesReader, CloudEventPayload, EventBuilder, EventBuilderV10, TypedEvent};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use url::Url;

// Define your domain events as regular Rust structs
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserCreated {
    user_id: String,
    email: String,
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OrderPlaced {
    order_id: String,
    user_id: String,
    total_amount: f64,
}

// Implement CloudEventPayload to associate each type with a CloudEvents type identifier
impl CloudEventPayload for UserCreated {
    const EVENT_TYPE: &'static str = "com.example.users.created";
}

impl CloudEventPayload for OrderPlaced {
    const EVENT_TYPE: &'static str = "com.example.orders.placed";
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a UserCreated event with full type safety
    let user_created = UserCreated {
        user_id: "user-123".to_string(),
        email: "alice@example.com".to_string(),
        name: "Alice".to_string(),
    };

    let event = EventBuilderV10::new()
        .id("event-001")
        .source(Url::parse("https://api.example.com")?)
        .ty("com.example.users.created")
        .data("application/json", serde_json::to_value(&user_created)?)
        .build()?;

    // Convert to typed event - validates the type and deserializes the data
    let typed_event: TypedEvent<UserCreated> = event.try_into()?;

    // Now you have full type safety!
    println!(
        "New user created: {} ({})",
        typed_event.data().name,
        typed_event.data().email
    );
    println!("CloudEvent ID: {}", typed_event.event().id());

    // Example: Processing events from different sources
    let order = OrderPlaced {
        order_id: "order-456".to_string(),
        user_id: "user-123".to_string(),
        total_amount: 99.99,
    };

    let order_event = EventBuilderV10::new()
        .id("event-002")
        .source(Url::parse("https://api.example.com")?)
        .ty("com.example.orders.placed")
        .data("application/json", serde_json::to_value(&order)?)
        .build()?;

    // Convert to OrderPlaced - would fail if type doesn't match
    let typed_order: TypedEvent<OrderPlaced> = order_event.try_into()?;

    println!(
        "Order placed: {} for user {}",
        typed_order.data().order_id,
        typed_order.data().user_id
    );
    println!("Total: ${}", typed_order.data().total_amount);

    // Example: Type mismatch handling
    let wrong_type_event = EventBuilderV10::new()
        .id("event-003")
        .source(Url::parse("https://api.example.com")?)
        .ty("com.example.orders.placed") // This says it's an OrderPlaced
        .data("application/json", serde_json::to_value(&user_created)?) // But the data is UserCreated
        .build()?;

    match TryInto::<TypedEvent<UserCreated>>::try_into(wrong_type_event) {
        Ok(_) => println!("This shouldn't happen"),
        Err(e) => println!("Expected error: {}", e),
    }

    Ok(())
}
