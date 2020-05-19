# CloudEvents SDK Rust [![Crates badge]][crates.io] [![Docs badge]][docs.rs] 

Work in progress SDK for [CloudEvents](https://github.com/cloudevents/spec).

Note: All APIs are considered unstable.

## Spec support

|                               |  [v0.3](https://github.com/cloudevents/spec/tree/v0.3) | [v1.0](https://github.com/cloudevents/spec/tree/v1.0) |
| :---------------------------: | :----------------------------------------------------------------------------: | :---------------------------------------------------------------------------------: |
| CloudEvents Core              | :heavy_check_mark: | :heavy_check_mark: |
| AMQP Protocol Binding         | :x: | :x:  |
| AVRO Event Format             | :x: | :x: |
| HTTP Protocol Binding         | :heavy_check_mark: | :heavy_check_mark: |
| JSON Event Format             | :heavy_check_mark: | :heavy_check_mark: |
| Kafka Protocol Binding        | :x: | :x: |
| MQTT Protocol Binding         | :x: | :x: |
| NATS Protocol Binding         | :x: | :x: |
| Web hook                      | :x: | :x: |

## Modules

* `cloudevents-sdk`: Provides Event data structure, JSON Event format implementation. This module is tested to work with GNU libc, WASM and musl toolchains.
* `cloudevents-sdk-actix-web`: Integration with [Actix Web](https://github.com/actix/actix-web).
* `cloudevents-sdk-reqwest`: Integration with [reqwest](https://github.com/seanmonstar/reqwest).

## Get Started

To get started, add the dependency to `Cargo.toml`:

```toml
cloudevents-sdk = "0.1.0"
```

Now you can start creating events:

```rust
use cloudevents::EventBuilder;
use url::Url;

let event = EventBuilder::v03()
    .id("aaa")
    .source(Url::parse("http://localhost").unwrap())
    .ty("example.demo")
    .build();
```

Checkout the examples using our integrations with `actix-web` and `reqwest` to learn how to send and receive events:

* [Actix Web Example](example-projects/actix-web-example)
* [Reqwest/WASM Example](example-projects/reqwest-wasm-example)

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