# Distributed tracing example

This example uses `tracing` and OpenTelemetry's W3C Trace Context propagator
to put the current instrumented span in the CloudEvents
[`traceparent` and `tracestate` extensions](https://github.com/cloudevents/spec/blob/main/cloudevents/extensions/distributed-tracing.md).
The values are strings, not bincode-serialized span data. `tracestate` is
included only when present. An OpenTelemetry layer and a valid active span
are required: `tracing` alone does not generate W3C trace IDs.

Run the producer and consumer as separate processes:

```sh
cargo run --manifest-path example-projects/tracing-example/Cargo.toml --features tracing -- produce \
  | cargo run --manifest-path example-projects/tracing-example/Cargo.toml --features tracing -- consume
```

The producer publishes a structured JSON CloudEvent to standard output; the
consumer reads it from standard input and prints its trace and span IDs to
standard error. To inspect the event, run the producer by itself. A real
application can send the same event with an SDK protocol binding, such as
Kafka, instead of the pipe.

## Consumer span lifecycle and context handling

The consumer extracts the trace context from the event and sets it as the parent
of its own span. The span remains active during event processing, ensuring all
work within the consumer is attributed to the correct trace.

If the event lacks a valid `traceparent` extension, the consumer falls back to
creating a new root span. This approach works well for single-message delivery.
For batch processing or when another active tracing context exists, consider
using [span links](https://opentelemetry.io/docs/specs/otel/trace/api/#add-links)
instead of parent-child relationships; OpenTelemetry's messaging semantic
conventions generally favor this pattern.

## Multi-hop delivery

For multi-hop delivery, intermediaries must preserve the original event
extensions rather than replacing them with each hop's context. Transport
tracing headers, when used, are separate from these extensions; for a single
hop they must carry the same trace information.

## Exporting traces

The example creates local spans but does not configure an exporter. Configure
an OpenTelemetry exporter and collector in both services to see the spans in
Jaeger or Tempo, and use matching OpenTelemetry crate versions in your
application. Do not shut down your tracer provider until pending spans have
been exported.
