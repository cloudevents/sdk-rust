//! This library provides Mqtt protocol bindings for CloudEvents using the [paho.mqtt.rust](https://github.com/eclipse/paho.mqtt.rust) library.\\
#[macro_use]
mod headers;
mod mqtt_consumer_record;
mod mqtt_producer_record;

pub use mqtt_consumer_record::record_to_event;
pub use mqtt_consumer_record::ConsumerMessageDeserializer;
pub use mqtt_consumer_record::MessageExt;

pub use headers::MqttVersion;
pub use mqtt_producer_record::MessageBuilderExt;
pub use mqtt_producer_record::MessageRecord;
