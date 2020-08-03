use chrono::prelude::*;
use clap::{App, Arg};
use env_logger::fmt::Formatter;
use env_logger::Builder;
use futures::StreamExt;
use log::{info, warn, LevelFilter, Record};
use serde_json::json;
use std::io::Write;
use std::str::FromStr;
use std::thread;
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
use rdkafka::util::get_rdkafka_version;

// run a kafka lense: docker run --rm --net=host -e ADV_HOST=localhost -e SAMPLEDATA=0 lensesio/fast-data-dev

pub fn setup_logger(log_thread: bool, rust_log: Option<&str>) {
    let output_format = move |formatter: &mut Formatter, record: &Record| {
        let thread_name = if log_thread {
            format!("(t: {}) ", thread::current().name().unwrap_or("unknown"))
        } else {
            "".to_string()
        };

        let local_time: DateTime<Local> = Local::now();
        let time_str = local_time.format("%H:%M:%S%.3f").to_string();
        write!(
            formatter,
            "{} {}{} - {} - {}\n",
            time_str,
            thread_name,
            record.level(),
            record.target(),
            record.args()
        )
    };

    let mut builder = Builder::new();
    builder
        .format(output_format)
        .filter(None, LevelFilter::Info);

    rust_log.map(|conf| builder.parse_filters(conf));

    builder.init();
}

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

// A type alias with your custom consumer can be created for convenience.
type LoggingConsumer = StreamConsumer<CustomContext>;

async fn consume_and_print(brokers: &str, group_id: &str, topics: &[&str]) {
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
            Err(e) => warn!("Kafka error: {}", e),
            Ok(m) => {
                let event = m.into_event().unwrap();
                info!("Received Event: {:#?}", event);

                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
        };
    }
}

#[tokio::main]
async fn consumer_example(log: Option<&str>, brokers: &str, group_id: &str, topics: Vec<&str>) {
    setup_logger(true, log);

    let (version_n, version_s) = get_rdkafka_version();
    info!("rd_kafka_version: 0x{:08x}, {}", version_n, version_s);

    consume_and_print(brokers, group_id, &topics).await
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
                .id("0001")
                .ty("example.test")
                .source(Url::from_str("http://localhost/").unwrap())
                .data("application/json", json!({"hello": "world"}))
                .extension("someint", "10")
                .build()
                .unwrap();

            info!("Sending event: {:#?}", event);

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
            info!("Delivery status for message {} received", i);
            delivery_status
        })
        .collect::<Vec<_>>();

    // This loop will wait until all delivery statuses have been received.
    for future in futures {
        info!("Future completed. Result: {:?}", future.await);
    }
}

#[tokio::main]
async fn producer_example(log: Option<&str>, brokers: &str, topic: &str) {
    setup_logger(true, log);

    let (version_n, version_s) = get_rdkafka_version();
    info!("rd_kafka_version: 0x{:08x}, {}", version_n, version_s);

    produce(brokers, topic).await;
}

fn main() {
    let selector = App::new("Main Application")
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
        .arg(
            Arg::with_name("log-conf")
                .long("log-conf")
                .help("Configure the logging format (example: 'rdkafka=trace')")
                .takes_value(true),
        )
        .get_matches();

    match selector.value_of("mode").unwrap() {
        "producer" => producer_example(
            selector.value_of("log-conf"),
            selector.value_of("brokers").unwrap(),
            selector.value_of("topic").unwrap(),
        ),
        "consumer" => consumer_example(
            selector.value_of("log-conf"),
            selector.value_of("brokers").unwrap(),
            selector.value_of("group-id").unwrap(),
            selector.values_of("topics").unwrap().collect::<Vec<&str>>(),
        ),
        _ => (),
    };
}
