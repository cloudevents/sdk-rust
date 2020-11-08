use clap::{App, Arg};
use std::process;
use futures::executor::block_on;
use paho_mqtt as mqtt;
use tokio::time::Duration;
use serde_json::json;
use std::option::Option::Some;
use tokio::stream::StreamExt;
use cloudevents::{EventBuilderV10, EventBuilder};
use cloudevents_sdk_mqtt::{MessageRecord, MessageBuilderExt, MqttVersion, MessageExt};

fn consume_v3(broker: &str, topic_name: &str) {

    let create_opts =   mqtt::CreateOptionsBuilder::new()
        .server_uri(broker)
        .client_id("rust_async_consumer")
        .finalize();

    let mut cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|e| {
        println!("Error creating the client: {:?}", e);
        process::exit(1);
    });

    if let Err(err) = block_on(async {
        let mut strm = cli.get_stream(25);

        let conn_opts = mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(20))
            .clean_session(false)
            .finalize();

        println!("Connecting to the MQTT server...");
        cli.connect(conn_opts).await?;

        println!("Subscribing to topics: {:?}", topic_name);
        cli.subscribe(topic_name, 1).await?;

        println!("Waiting for messages...");

        while let Some(msg_opt) = strm.next().await {
            if let Some(msg) = msg_opt {
                let event = msg.to_event(MqttVersion::V3_1_1).unwrap();
                println!("Received Event: {:#?}", event);
            }
            else {
                // A "None" means we were disconnected. Try to reconnect...
                println!("Lost connection. Attempting reconnect.");
                while let Err(_err) = cli.reconnect().await {
                    // For tokio use: tokio::time::delay_for()
                    tokio::time::delay_for(Duration::from_millis(1000)).await;
                }
            }
        }

        Ok::<(), mqtt::Error>(())
    }) {
        eprintln!("{}", err);
    }
}

fn consume_v5(broker: &str, topic_name: &str) {

    let create_opts =   mqtt::CreateOptionsBuilder::new()
        .server_uri(broker)
        .client_id("rust_async_consumer")
        .mqtt_version(5)
        .finalize();

    let mut cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|e| {
        println!("Error creating the client: {:?}", e);
        process::exit(1);
    });

    if let Err(err) = block_on(async {
        let mut strm = cli.get_stream(25);

        let conn_opts = mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(20))
            .clean_session(false)
            .mqtt_version(5)
            .finalize();

        println!("Connecting to the MQTT server...");
        cli.connect(conn_opts).await?;

        println!("Subscribing to topics: {:?}", topic_name);
        cli.subscribe(topic_name, 1).await?;

        println!("Waiting for messages...");

        while let Some(msg_opt) = strm.next().await {
            if let Some(msg) = msg_opt {
                let event = msg.to_event(MqttVersion::V5).unwrap();
                println!("Received Event: {:#?}", event);
            }
            else {
                // A "None" means we were disconnected. Try to reconnect...
                println!("Lost connection. Attempting reconnect.");
                while let Err(_err) = cli.reconnect().await {
                    // For tokio use: tokio::time::delay_for()
                    tokio::time::delay_for(Duration::from_millis(1000)).await;
                }
            }
        }

        Ok::<(), mqtt::Error>(())
    }) {
        eprintln!("{}", err);
    }
}

fn produce_v3(broker: &str, topic_name: &str) {
    env_logger::init();

    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(broker)
        .finalize();

    let cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|_err| {
        process::exit(1);
    });

    if let Err(err) = block_on(async {
        let conn_opts = mqtt::ConnectOptions::new();

        cli.connect(conn_opts).await?;

        println!("Publishing a message on the topic");

        let event = EventBuilderV10::new()
            .id("1".to_string())
            .ty("example.test")
            .source("http://localhost/")
            .data("application/json", json!({"hello": "world"}))
            .build()
            .unwrap();

        let message_record =
            MessageRecord::from_event(event, MqttVersion::V3_1_1).expect("error while serializing the event");

        // Create a message and publish it
        let msg = mqtt::MessageBuilder::new()
            .topic(topic_name)
            .message_record(&message_record)
            .qos(1)
            .finalize();

        cli.publish(msg).await?;

        cli.disconnect(None).await?;

        Ok::<(), mqtt::Error>(())
    }) {
        eprintln!("{}", err);
    }
}

fn produce_v5(broker: &str, topic_name: &str) {
    env_logger::init();

    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(broker)
        .mqtt_version(5)
        .finalize();

    let cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|_err| {
        process::exit(1);
    });

    if let Err(err) = block_on(async {
        let conn_opts = mqtt::ConnectOptionsBuilder::new()
            .mqtt_version(5)
            .finalize();

        cli.connect(conn_opts).await?;

        println!("Publishing a message on the topic");

        let event = EventBuilderV10::new()
            .id("1".to_string())
            .ty("example.test")
            .source("http://localhost/")
            .data("application/json", json!({"hello": "world"}))
            .build()
            .unwrap();

        let message_record =
            MessageRecord::from_event(event, MqttVersion::V5).expect("error while serializing the event");

        // Create a message and publish it
        let msg = mqtt::MessageBuilder::new()
            .topic(topic_name)
            .message_record(&message_record)
            .qos(1)
            .finalize();

        cli.publish(msg).await?;

        cli.disconnect(None).await?;

        Ok::<(), mqtt::Error>(())
    }) {
        eprintln!("{}", err);
    }
}

fn main() {
    let selector = App::new("CloudEvents Mqtt Example")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or(""))
        .about("select consumer or producer")
        .arg(
            Arg::with_name("mode")
                .long("mode")
                .help("enter \"consmer\" or \"producer\"")
                .takes_value(true)
                .possible_values(&["consumerV3", "producerV3", "consumerV5", "producerV5"])
                .required(true),
        )
        .arg(
            Arg::with_name("topic")
                .long("topic")
                .help("Mqtt topic")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("broker")
                .short("b")
                .long("broker")
                .help("Broker list in mqtt format")
                .takes_value(true)
                .default_value("tcp://localhost:1883"),
        )
        .get_matches();


    match selector.value_of("mode").unwrap() {
        "producerV3" => {
            produce_v3(
                selector.value_of("broker").unwrap(),
                selector.value_of("topic").unwrap(),
            )
        }
        "consumerV3" => {
            consume_v3(
                selector.value_of("broker").unwrap(),
                selector.value_of("topic").unwrap(),
            )
        }
        "producerV5" => {
            produce_v5(
                selector.value_of("broker").unwrap(),
                selector.value_of("topic").unwrap(),
            )
        }
        "consumerV5" => {
            consume_v5(
                selector.value_of("broker").unwrap(),
                selector.value_of("topic").unwrap(),
            )
        }
        _ => (),
    }
}