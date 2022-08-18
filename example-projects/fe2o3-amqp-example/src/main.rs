//! AMQP 1.0 binding example
//!
//! You need a running AMQP 1.0 broker to try out this example.
//! With docker: docker run -it --rm -e ARTEMIS_USERNAME=guest -e ARTEMIS_PASSWORD=guest -p 5672:5672 vromero/activemq-artemis

use cloudevents::{
    binding::fe2o3_amqp::EventMessage, message::MessageDeserializer, Event, EventBuilder,
    EventBuilderV10, AttributesReader, event::ExtensionValue,
};
use fe2o3_amqp::{types::messaging::Message, Connection, Receiver, Sender, Session};
use serde_json::{json, from_slice, from_str};

type BoxError = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, BoxError>;

const EXAMPLE_TYPE: &str = "example.test";
const EXAMPLE_SOURCE: &str = "localhost";
const EXTENSION_NAME: &str = "ext-name";
const EXTENSION_VALUE: &str = "AMQP";

async fn send_binary_event(sender: &mut Sender, i: usize, value: serde_json::Value) -> Result<()> {
    let event = EventBuilderV10::new()
        .id(i.to_string())
        .ty(EXAMPLE_TYPE)
        .source(EXAMPLE_SOURCE)
        .extension(EXTENSION_NAME, EXTENSION_VALUE)
        .data("application/json", value)
        .build()?;
    let event_message = EventMessage::from_binary_event(event)?;
    let message = Message::from(event_message);
    sender.send(message).await?.accepted_or("not accepted")?;
    Ok(())
}

async fn send_structured_event(sender: &mut Sender, i: usize, value: serde_json::Value) -> Result<()> {
    let event = EventBuilderV10::new()
        .id(i.to_string())
        .ty("example.test")
        .source("localhost")
        .extension("ext-name", "AMQP")
        .data("application/json", value)
        .build()?;
    let event_message = EventMessage::from_structured_event(event)?;
    let message = Message::from(event_message);
    sender.send(message).await?.accepted_or("not accepted")?;
    Ok(())
}

async fn recv_event(receiver: &mut Receiver) -> Result<Event> {
    let delivery = receiver.recv().await?;
    receiver.accept(&delivery).await?;

    let event_message = EventMessage::from(delivery.into_message());
    let event = MessageDeserializer::into_event(event_message)?;
    Ok(event)
}

fn convert_data_into_json_value(data: &cloudevents::Data) -> Result<serde_json::Value> {
    let value = match data {
        cloudevents::Data::Binary(bytes) => from_slice(bytes)?,
        cloudevents::Data::String(s) => from_str(s)?,
        cloudevents::Data::Json(value) => value.clone(),
    };
    Ok(value)
}

#[tokio::main]
async fn main() {
    let mut connection =
        Connection::open("cloudevents-sdk-rust", "amqp://guest:guest@localhost:5672")
            .await
            .unwrap();
    let mut session = Session::begin(&mut connection).await.unwrap();
    let mut sender = Sender::attach(&mut session, "sender", "q1").await.unwrap();
    let mut receiver = Receiver::attach(&mut session, "receiver", "q1")
        .await
        .unwrap();

    let expected = json!({"hello": "world"});

    // Binary content mode
    send_binary_event(&mut sender, 1, expected.clone()).await.unwrap();
    let event = recv_event(&mut receiver).await.unwrap();
    let value = convert_data_into_json_value(event.data().unwrap()).unwrap();
    assert_eq!(event.id(), "1");
    assert_eq!(event.ty(), EXAMPLE_TYPE);
    assert_eq!(event.source(), EXAMPLE_SOURCE);
    match event.extension(EXTENSION_NAME).unwrap() {
        ExtensionValue::String(value) => assert_eq!(value, EXTENSION_VALUE),
        _ => panic!("Expect a String"),
    }
    assert_eq!(value, expected);

    // Structured content mode
    send_structured_event(&mut sender, 2, expected.clone()).await.unwrap();
    let event = recv_event(&mut receiver).await.unwrap();
    let value = convert_data_into_json_value(event.data().unwrap()).unwrap();
    assert_eq!(event.id(), "2");
    assert_eq!(event.ty(), EXAMPLE_TYPE);
    assert_eq!(event.source(), EXAMPLE_SOURCE);
    match event.extension(EXTENSION_NAME).unwrap() {
        ExtensionValue::String(value) => assert_eq!(value, EXTENSION_VALUE),
        _ => panic!("Expect a String"),
    }
    assert_eq!(value, expected);

    sender.close().await.unwrap();
    receiver.close().await.unwrap();
    session.end().await.unwrap();
    connection.close().await.unwrap();
}
