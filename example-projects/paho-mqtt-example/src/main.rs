use clap::{App, Arg};
use std::process;
use futures::executor::block_on;
use paho_mqtt as mqtt;
use tokio::time::Duration;
use serde_json::json;
use std::option::Option::Some;
use tokio::stream::StreamExt;
use cloudevents::{EventBuilderV10, EventBuilder};
use cloudevents_sdk_paho_mqtt::{MessageBuilderExt, MessageExt, MqttVersion};
use paho_mqtt::AsyncClient;

async fn consume_v3(cli: &mut AsyncClient, topic_name: &str) -> Result<(), mqtt::Error> {
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
            let event = msg.to_event().unwrap();
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
}

async fn consume_v5(cli: &mut AsyncClient, topic_name: &str) -> Result<(), mqtt::Error> {
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
            let event = msg.to_event().unwrap();
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
}

async fn produce_v3(cli: &AsyncClient, topic_name: &str) -> Result<(), mqtt::Error> {
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

    // Create a message and publish it
    let msg = mqtt::MessageBuilder::new()
        .topic(topic_name)
        .event(event, MqttVersion::MQTT_3)
        .qos(1)
        .finalize();

    cli.publish(msg).await?;

    cli.disconnect(None).await?;

    Ok::<(), mqtt::Error>(())
}

async fn produce_v5(cli: &AsyncClient, topic_name: &str) -> Result<(), mqtt::Error> {
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

    // Create a message and publish it
    let msg = mqtt::MessageBuilder::new()
        .topic(topic_name)
        .event(event, MqttVersion::MQTT_5)
        .qos(1)
        .finalize();

    cli.publish(msg).await?;

    cli.disconnect(None).await?;

    Ok::<(), mqtt::Error>(())
}

fn main() {
    let selector = App::new("CloudEvents Mqtt Example")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or(""))
        .about("select consumer or producer")
        .arg(
            Arg::with_name("mode")
                .long("mode")
                .help("enter \"producerV3\" or \"producerV5\" or \"consumerV3\" or \"consumerV5\"")
                .takes_value(true)
                .possible_values(&["producerV3", "producerV5", "consumerV3", "consumerV5"])
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
            env_logger::init();

            let create_opts = mqtt::CreateOptionsBuilder::new()
                .server_uri(selector.value_of("broker").unwrap())
                .finalize();

            let cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|_err| {
                process::exit(1);
            });

            if let Err(err) = block_on(produce_v3(
                &cli, selector.value_of("topic").unwrap(),
            )) {
                eprintln!("{}", err);
            }
        }
        "consumerV3" => {
            let create_opts =   mqtt::CreateOptionsBuilder::new()
                .server_uri(selector.value_of("broker").unwrap())
                .client_id("rust_async_consumer")
                .finalize();

            let mut cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|e| {
                println!("Error creating the client: {:?}", e);
                process::exit(1);
            });

            if let Err(err) = block_on(consume_v3(
                &mut cli, selector.value_of("topic").unwrap(),
            )) {
                eprintln!("{}", err);
            }
        }
        "producerV5" => {
            env_logger::init();

            let create_opts = mqtt::CreateOptionsBuilder::new()
                .server_uri(selector.value_of("broker").unwrap())
                .mqtt_version(5)
                .finalize();

            let cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|_err| {
                process::exit(1);
            });

            if let Err(err) = block_on(produce_v5(
                &cli, selector.value_of("topic").unwrap(),
            )) {
                eprintln!("{}", err);
            }
        }
        "consumerV5" => {
            let create_opts =   mqtt::CreateOptionsBuilder::new()
                .server_uri(selector.value_of("broker").unwrap())
                .client_id("rust_async_consumer")
                .mqtt_version(5)
                .finalize();

            let mut cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|e| {
                println!("Error creating the client: {:?}", e);
                process::exit(1);
            });

            if let Err(err) = block_on(consume_v5(
                &mut cli,
                selector.value_of("topic").unwrap(),
            )) {
                eprintln!("{}", err);
            }
        }
        _ => (),
    }
}