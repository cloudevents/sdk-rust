//! AMQP 1.0 binding example
//!
//! You need a running AMQP 1.0 broker to try out this example.
//! With docker: docker run -it --rm -e ARTEMIS_USERNAME=guest -e ARTEMIS_PASSWORD=guest -p 5672:5672 vromero/activemq-artemis

use cloudevents::{
    binding::fe2o3_amqp::EventMessage, message::MessageDeserializer, Event, EventBuilder,
    EventBuilderV10,
};
use fe2o3_amqp::{types::messaging::Message, Connection, Receiver, Sender, Session};
use serde_json::json;

type BoxError = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, BoxError>;

async fn send_event(sender: &mut Sender, i: usize) -> Result<()> {
    let event = EventBuilderV10::new()
        .id(i.to_string())
        .ty("example.test")
        .source("localhost")
        .extension("ext-name", "AMQP")
        .data("application/json", json!({"hello": "world"}))
        .build()?;
    let event_message = EventMessage::from_binary_event(event)?;
    let message = Message::from(event_message);
    sender.send(message).await?.accepted_or("not accepted")?;
    Ok(())
}

async fn recv_event(receiver: &mut Receiver) -> Result<Event> {
    use fe2o3_amqp::types::primitives::Value;

    let delivery = receiver.recv::<Value>().await?;
    receiver.accept(&delivery).await?;

    let event_message = EventMessage::from(delivery.into_message());
    let event = MessageDeserializer::into_event(event_message)?;
    Ok(event)
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

    send_event(&mut sender, 1).await.unwrap();
    let event = recv_event(&mut receiver).await.unwrap();
    println!("{:?}", event);

    sender.close().await.unwrap();
    receiver.close().await.unwrap();
    session.end().await.unwrap();
    connection.close().await.unwrap();
}
