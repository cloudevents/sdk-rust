# CloudEvents SDK Rust [![Crates badge]][crates.io] [![Docs badge]][docs.rs] 

This project implements the [CloudEvents](https://cloudevents.io/) Spec for Rust.

Note: This project is WIP under active development, hence all APIs are considered unstable.

## Spec support

|                               |  [v0.3](https://github.com/cloudevents/spec/tree/v0.3) | [v1.0](https://github.com/cloudevents/spec/tree/v1.0) |
| :---------------------------: | :----------------------------------------------------------------------------: | :---------------------------------------------------------------------------------: |
| CloudEvents Core              | ✓ | ✓ |
| AMQP Protocol Binding         | ✕ | ✕ |
| AVRO Event Format             | ✕ | ✕ |
| HTTP Protocol Binding         | ✓ | ✓ |
| JSON Event Format             | ✓ | ✓ |
| Kafka Protocol Binding        | ✓ | ✓ |
| MQTT Protocol Binding         | ✕ | ✕ |
| NATS Protocol Binding         | ✕ | ✕ |
| Web hook                      | ✕ | ✕ |

## Crates

* `cloudevents-sdk`: Provides Event data structure, JSON Event format implementation. This module is tested to work with GNU libc, WASM and musl toolchains.
* `cloudevents-sdk-actix-web`: Integration with [Actix Web](https://github.com/actix/actix-web).
* `cloudevents-sdk-reqwest`: Integration with [reqwest](https://github.com/seanmonstar/reqwest).
* `cloudevents-sdk-rdkafka`: Integration with [rust-rdkafka](https://fede1024.github.io/rust-rdkafka).

## Get Started

To get started, add the dependency to `Cargo.toml`:

```toml
cloudevents-sdk = "0.2.0"
```

Now you can start creating events:

```rust
use cloudevents::{EventBuilder, EventBuilderV10};
use url::Url;

let event = EventBuilderV10::new()
    .id("aaa")
    .source(Url::parse("http://localhost").unwrap())
    .ty("example.demo")
    .build()
    .unwrap();
```

Checkout the examples using our integrations to learn how to send and receive events:

* [Actix Web Example](example-projects/actix-web-example)
* [Reqwest/WASM Example](example-projects/reqwest-wasm-example)
* [Kafka Example](example-projects/rdkafka-example)

## Development & Contributing

If you're interested in contributing to sdk-rust, look at [Contributing documentation](CONTRIBUTING.md)

## Community

- There are bi-weekly calls immediately following the
  [Serverless/CloudEvents call](https://github.com/cloudevents/spec#meeting-time)
  at 9am PT (US Pacific). Which means they will typically start at 10am PT, but
  if the other call ends early then the SDK call will start early as well. See
  the
  [CloudEvents meeting minutes](https://docs.google.com/document/d/1OVF68rpuPK5shIHILK9JOqlZBbfe91RNzQ7u_P7YCDE/edit#)
  to determine which week will have the call.
- Slack: #cloudeventssdk (or #cloudevents-sdk-rust) channel under
  [CNCF's Slack workspace](https://slack.cncf.io/).
- Email: https://lists.cncf.io/g/cncf-cloudevents-sdk
- Contact for additional information: Francesco Guardiani (`@slinkydeveloper`
  on slack).

[Crates badge]: https://img.shields.io/crates/v/cloudevents-sdk.svg
[crates.io]: https://crates.io/crates/cloudevents-sdk
[Docs badge]: https://docs.rs/cloudevents-sdk/badge.svg
[docs.rs]: https://docs.rs/cloudevents-sdk