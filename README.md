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

## Crate Structure

The core modules include definitions for the `Event` and
`EventBuilder` data structures, JSON serialization rules, and a
mechanism to support various Protocol Bindings, each of which is
enabled by a specific [feature flag]:

* `actix`: Integration with [actix](https://actix.rs/).
* `warp`: Integration with [warp](https://github.com/seanmonstar/warp/).
* `reqwest`: Integration with [reqwest](https://github.com/seanmonstar/reqwest).
* `rdkafka`: Integration with [rdkafka](https://fede1024.github.io/rust-rdkafka).

This crate is continuously tested to work with GNU libc, WASM and musl
toolchains.

## Get Started

To get started, add the dependency to `Cargo.toml`, optionally
enabling your Protocol Binding of choice:

```toml
[dependencies]
cloudevents-sdk = { version = "0.3.1", features = ["actix"] }
```

Now you can start creating events:

```rust
use cloudevents::{EventBuilder, EventBuilderV10};
use url::Url;

let event = EventBuilderV10::new()
    .id("aaa")
    .source(Url::parse("http://localhost").unwrap())
    .ty("example.demo")
    .build()?;
```

Checkout the examples using our integrations to learn how to send and receive events:

* [Actix Web Example](example-projects/actix-web-example)
* [Reqwest/WASM Example](example-projects/reqwest-wasm-example)
* [Kafka Example](example-projects/rdkafka-example)
* [Warp Example](example-projects/warp-example)

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

Each SDK may have its own unique processes, tooling and guidelines, common
governance related material can be found in the
[CloudEvents `community`](https://github.com/cloudevents/spec/tree/master/community)
directory. In particular, in there you will find information concerning
how SDK projects are
[managed](https://github.com/cloudevents/spec/blob/master/community/SDK-GOVERNANCE.md),
[guidelines](https://github.com/cloudevents/spec/blob/master/community/SDK-maintainer-guidelines.md)
for how PR reviews and approval, and our
[Code of Conduct](https://github.com/cloudevents/spec/blob/master/community/GOVERNANCE.md#additional-information)
information.

[Crates badge]: https://img.shields.io/crates/v/cloudevents-sdk.svg
[crates.io]: https://crates.io/crates/cloudevents-sdk
[Docs badge]: https://docs.rs/cloudevents-sdk/badge.svg
[docs.rs]: https://docs.rs/cloudevents-sdk
[feature flag]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-features-section
