use clap::{App, Arg};
use futures::StreamExt;
use serde_json::json;
use std::str::FromStr;
use url::Url;

use cloudevents::{EventBuilder, EventBuilderV10};
use cloudevents_sdk_rdkafka::{BorrowedMessageExt, EventExt, FutureRecordExt};

use rdkafka::client::ClientContext;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{CommitMode, Consumer, ConsumerContext, Rebalance};
use rdkafka::error::KafkaResult;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::topic_partition_list::TopicPartitionList;

// run a Kafka lense or a Kafka Docker container to try out this example

// A context can be used to change the behavior of producers and consumers by adding callbacks
// that will be executed by librdkafka.
// This particular context sets up custom callbacks to log rebalancing events.
struct CustomContext;

impl ClientContext for CustomContext {}

impl ConsumerContext for CustomContext {
    fn pre_rebalance(&self, rebalance: &Rebalance) {
        println!("Pre rebalance {:?}", rebalance);
    }

    fn post_rebalance(&self, rebalance: &Rebalance) {
        println!("Post rebalance {:?}", rebalance);
    }

    fn commit_callback(&self, result: KafkaResult<()>, _offsets: &TopicPartitionList) {
        println!("Committing offsets: {:?}", result);
    }
}

async fn consume(brokers: &str, group_id: &str, topics: &[&str]) {
    let context = CustomContext;

    let consumer: LoggingConsumer = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        //.set("statistics.interval.ms", "30000")
        //.set("auto.offset.reset", "smallest")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create_with_context(context)
        .expect("Consumer creation failed");

    consumer
        .subscribe(&topics.to_vec())
        .expect("Can't subscribe to specified topics");

    // consumer.start() returns a stream. The stream can be used ot chain together expensive steps,
    // such as complex computations on a thread pool or asynchronous IO.
    let mut message_stream = consumer.start();

    while let Some(message) = message_stream.next().await {
        match message {
            Err(e) => println!("Kafka error: {}", e),
            Ok(m) => {
                let event = m.from_event().unwrap();
                println!("Received Event: {:#?}", event);

                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
        };
    }
}

async fn produce(brokers: &str, topic_name: &str) {
    let producer: &FutureProducer = &ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");

    // This loop is non blocking: all messages will be sent one after the other, without waiting
    // for the results.
    let futures = (0..5)
        .map(|i| async move {
            // The send operation on the topic returns a future, which will be
            // completed once the result or failure from Kafka is received.
            let event = EventBuilderV10::new()
                .id(i)
                .ty("example.test")
                .source("http://localhost/")
                .data("application/json", json!({"hello": "world"}))
                .build()
                .unwrap();

            println!("Sending event: {:#?}", event);

            let delivery_status = producer
                .send(
                    FutureRecord::to(topic_name)
                        .event(&event.serialize_event().unwrap())
                        .unwrap()
                        .key(&format!("Key {}", i)),
                    0,
                )
                .await;

            // This will be executed when the result is received.
            println!("Delivery status for message {} received", i);
            delivery_status
        })
        .collect::<Vec<_>>();

    // This loop will wait until all delivery statuses have been received.
    for future in futures {
        println!("Future completed. Result: {:?}", future.await);
    }
}

#[tokio::main]
async fn main() {
    let selector = App::new("CloudEvents Kafka Example")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or(""))
        .about("select consumer or producer")
        .arg(
            Arg::with_name("mode")
                .long("mode")
                .help("enter \"consmer\" or \"producer\"")
                .takes_value(true)
                .possible_values(&["consumer", "producer"])
                .required(true),
        )
        .arg(
            Arg::with_name("topics")
                .long("topics")
                .help("Topic list")
                .takes_value(true)
                .multiple(true)
                .requires_if("consumer", "mode"),
        )
        .arg(
            Arg::with_name("topic")
                .long("topic")
                .help("Destination topic")
                .takes_value(true)
                .requires_if("producer", "mode"),
        )
        .arg(
            Arg::with_name("brokers")
                .short("b")
                .long("brokers")
                .help("Broker list in kafka format")
                .takes_value(true)
                .default_value("localhost:9092"),
        )
        .arg(
            Arg::with_name("group-id")
                .short("g")
                .long("group-id")
                .help("Consumer group id")
                .takes_value(true)
                .default_value("example_consumer_group_id"),
        )
        .get_matches();

    match selector.value_of("mode").unwrap() {
        "producer" => {
            produce(
                selector.value_of("brokers").unwrap(),
                selector.value_of("topic").unwrap(),
            )
            .await
        }
        "consumer" => consume(
            selector.value_of("brokers").unwrap(),
            selector.value_of("group-id").unwrap(),
            &selector.values_of("topics").unwrap().collect::<Vec<&str>>(),
        ).await,
        _ => (),
    };
}
