# CloudEvents SDK Rust - paho-mqtt [![Crates badge]][crates.io] [![Docs badge]][docs.rs]

Integration of [CloudEvents SDK](https://github.com/cloudevents/sdk-rust/) with [paho-mqtt](https://www.eclipse.org/paho/).

Look at [CloudEvents SDK README](https://github.com/cloudevents/sdk-rust/) for more info.

## Development & Contributing

If you're interested in contributing to sdk-rust, look at [Contributing documentation](../CONTRIBUTING.md)

## Community

## Sample usage

- Check the example [paho-mqtt-example](../example-projects/paho-mqtt-example)

### MQTT V3
- Start the MQTT V3 Consumer

```
run --package <package-name> --bin <binary-name> -- --mode consumerV3 --broker tcp://localhost:1883 --topic test
```

- Start the MQTT V3 Producer

```
run --package <package-name> --bin <binary-name> -- --broker tcp://localhost:1883 --topic test --mode producerV3
```

### MQTT V5
- Start the MQTT V5 Consumer

```
run --package <package-name> --bin <binary-name> -- --mode consumerV5 --broker tcp://localhost:1883 --topic test
```

- Start the MQTT V5 Producer

```
run --package <package-name> --bin <binary-name> -- --broker tcp://localhost:1883 --topic test --mode producerV5
```

