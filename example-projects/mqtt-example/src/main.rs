use cloudevents::{EventBuilder, EventBuilderV10};
use paho_mqtt as mqtt;
use std::{ process, time::Duration };
use serde_json::json;
use cloudevents::binding::paho_mqtt::{ MessageBuilderExt, MqttVersion, MessageExt };
use clap::{ Arg, Command };
use futures::executor::block_on;
use futures::StreamExt;
use paho_mqtt::AsyncClient;

async fn produce_v3(cli: &mqtt::AsyncClient, topic_name: &str) -> Result<(), mqtt::Error> {
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

    let msg = mqtt::MessageBuilder::new()
        .topic(topic_name)
        .event(event, MqttVersion::MQTT_3_1_1)
        .qos(1)
        .finalize();

    cli.publish(msg).await?;
    cli.disconnect(None).await?;
    Ok::<(), mqtt::Error>(())
}

async fn consume_v3(cli: &mut AsyncClient, topic_name: &str) -> Result<(), mqtt::Error> {
    let mut strm = Box::pin(cli.get_stream(25));

    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(true)
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
        } else {
            println!("Lost connection. Attempting reconnect.");
            while let Err(_err) = cli.reconnect().await {
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        }
    }
    Ok::<(), mqtt::Error>(())
}

async fn produce_v5(cli: &mqtt::AsyncClient, topic_name: &str) -> Result<(), mqtt::Error> {
    let conn_opts = mqtt::ConnectOptionsBuilder::new_v5()
        .clean_session(true)
        .clean_start(true)
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

    let msg = mqtt::MessageBuilder::new()
        .topic(topic_name)
        .event(event, MqttVersion::MQTT_5)
        .qos(1)
        .finalize();

    cli.publish(msg).await?;
    cli.disconnect(None).await?;
    Ok::<(), mqtt::Error>(())
}

async fn consume_v5(cli: &mut AsyncClient, topic_name: &str) -> Result<(), mqtt::Error> {
    let mut strm = Box::pin(cli.get_stream(25));

    let conn_opts = mqtt::ConnectOptionsBuilder::new_v5()
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(true)
        .clean_start(true)
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
        } else {
            println!("Lost connection. Attempting reconnect.");
            while let Err(_err) = cli.reconnect().await {
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        }
    }
    Ok::<(), mqtt::Error>(())
}

fn main() {
    let selector = Command::new("CloudEvents Mqtt Example")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or(""))
        .about("select consumer or producer")
        .arg(
            Arg::new("mode")
                .long("mode")
                .help("enter \"producerV5\" or \"consumerV5\"")
                .value_parser([
                    "producerV3",
                    "consumerV3",
                    "producerV5",
                    "consumerV5",
                ])
                .required(true)
        )
        .arg(
            Arg::new("topic")
                .long("topic")
                .help("Mqtt topic")
                .required(true)
        )
        .arg(
            Arg::new("broker")
                .long("broker")
                .help("Broker list in mqtt format")
                .default_value("tcp://localhost:1883")
        )
        .get_matches();

    match selector.get_one::<String>("mode").unwrap().as_str() {
        "producerV3" => {
            let create_opts = mqtt::CreateOptionsBuilder::new()
                .server_uri(selector.get_one::<String>("broker").unwrap())
                .finalize();

            let cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|_err| {
                process::exit(1);
            });

            if let Err(err) = block_on(produce_v3(
                &cli, selector.get_one::<String>("topic").unwrap()
            )) {
                eprintln!("{}", err);
            }
        },
        "consumerV3" => {
            let create_opts = mqtt::CreateOptionsBuilder::new()
                .server_uri(selector.get_one::<String>("broker").unwrap())
                .finalize();

            let mut cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|e| {
                println!("Error creating the client: {:?}", e);
                process::exit(1);
            });

            if let Err(err) = block_on(consume_v3(
                &mut cli, selector.get_one::<String>("topic").unwrap()
            )) {
                eprintln!("{}", err);
            }
        }
        "producerV5" => {
            let create_opts = mqtt::CreateOptionsBuilder::new()
                .server_uri(selector.get_one::<String>("broker").unwrap())
                .mqtt_version(mqtt::MQTT_VERSION_5)
                .finalize();

            let cli = mqtt::AsyncClient::new(create_opts)
                .unwrap_or_else(|_err| {process::exit(1);});

            if let Err(err) = block_on(produce_v5(
                &cli, selector.get_one::<String>("topic").unwrap()
            )) {
                eprintln!("{}", err);
            }
        }
        "consumerV5" => {
            let create_opts = mqtt::CreateOptionsBuilder::new()
                .server_uri(selector.get_one::<String>("broker").unwrap())
                .mqtt_version(mqtt::MQTT_VERSION_5)
                .finalize();

            let mut cli = mqtt::AsyncClient::new(create_opts)
                .unwrap_or_else(|e| {
                    println!("rror creating the client: {:?}", e);
                    process::exit(1);
                });

            if let Err(err) = block_on(consume_v5(
                &mut cli, selector.get_one::<String>("topic").unwrap(),
            )) {
                eprintln!("{}", err);
            }
        }
        _ => ()
    }
}