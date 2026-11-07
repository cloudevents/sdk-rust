use std::{collections::HashMap, error::Error, io};

use cloudevents::{event::ExtensionValue, Event, EventBuilder, EventBuilderV10};
use opentelemetry::{
    propagation::{Extractor, TextMapPropagator},
    trace::{TraceContextExt, TracerProvider as _},
};
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace::SdkTracerProvider};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::prelude::*;

struct EventExtractor<'a>(&'a Event);

impl Extractor for EventExtractor<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        match self.0.extension(key) {
            Some(ExtensionValue::String(value)) => Some(value),
            _ => None,
        }
    }

    fn keys(&self) -> Vec<&str> {
        self.0.iter_extensions().map(|(key, _)| key).collect()
    }
}

#[tracing::instrument]
fn create_event() -> Result<Event, Box<dyn Error>> {
    let mut carrier = HashMap::new();
    TraceContextPropagator::new().inject_context(&tracing::Span::current().context(), &mut carrier);
    let traceparent = carrier
        .remove("traceparent")
        .ok_or("no active OpenTelemetry trace context")?;

    let mut builder = EventBuilderV10::new()
        .id("example-1")
        .source("https://example.com/producer")
        .ty("example.created")
        .data("application/json", serde_json::json!({"message": "hello"}))
        .extension("traceparent", traceparent);
    if let Some(tracestate) = carrier
        .remove("tracestate")
        .filter(|value| !value.is_empty())
    {
        builder = builder.extension("tracestate", tracestate);
    }
    Ok(builder.build()?)
}

fn receive_event(event: &Event) -> Result<(), Box<dyn Error>> {
    let parent = TraceContextPropagator::new().extract(&EventExtractor(event));
    let span = if parent.span().span_context().is_valid() {
        let s = tracing::info_span!("receive_event");
        s.set_parent(parent)?;
        s
    } else {
        tracing::info_span!("receive_event")
    };
    let _guard = span.enter();
    
    process_event(event)?;
    Ok(())
}

fn process_event(_event: &Event) -> Result<(), Box<dyn Error>> {
    let context = tracing::Span::current().context();
    let span = context.span();
    let span_context = span.span_context();
    eprintln!(
        "Received trace {} span {}",
        span_context.trace_id(),
        span_context.span_id()
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let provider = SdkTracerProvider::builder().build();
    let tracer = provider.tracer("cloudevents-tracing-example");
    let subscriber =
        tracing_subscriber::registry().with(tracing_opentelemetry::layer().with_tracer(tracer));
    tracing::subscriber::set_global_default(subscriber)?;

    match std::env::args().nth(1).as_deref() {
        Some("produce") => println!("{}", serde_json::to_string(&create_event()?)?),
        Some("consume") => {
            let event: Event = serde_json::from_reader(io::stdin().lock())?;
            receive_event(&event)?;
        }
        _ => return Err("usage: tracing-example produce|consume".into()),
    }
    provider.shutdown()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::trace::TraceFlags;

    #[test]
    fn event_propagates_trace_context_across_serialization() {
        let provider = SdkTracerProvider::builder().build();
        let subscriber = tracing_subscriber::registry()
            .with(tracing_opentelemetry::layer().with_tracer(provider.tracer("test")));
        tracing::subscriber::with_default(subscriber, || {
            let carrier = HashMap::from([
                (
                    "traceparent".to_owned(),
                    "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01".to_owned(),
                ),
                ("tracestate".to_owned(), "vendor=value".to_owned()),
            ]);
            let parent = TraceContextPropagator::new().extract(&carrier);
            let span = tracing::info_span!("producer");
            span.set_parent(parent).unwrap();
            let _guard = span.enter();

            let event = create_event().unwrap();
            let json = serde_json::to_string(&event).unwrap();
            let received: Event = serde_json::from_str(&json).unwrap();
            let extracted = TraceContextPropagator::new().extract(&EventExtractor(&received));
            let extracted_span = extracted.span();
            let extracted_context = extracted_span.span_context();
            assert_eq!(
                extracted_context.trace_id().to_string(),
                "4bf92f3577b34da6a3ce929d0e0e4736"
            );
            assert_ne!(extracted_context.span_id().to_string(), "00f067aa0ba902b7");
            assert_eq!(extracted_context.trace_flags(), TraceFlags::SAMPLED);
            assert_eq!(extracted_context.trace_state().header(), "vendor=value");
            assert_eq!(
                received.extension("tracestate"),
                Some(&ExtensionValue::String("vendor=value".to_owned()))
            );

            let consumer = receive_event(&received);
            assert!(consumer.is_ok());
        });
        provider.shutdown().unwrap();
    }

    #[test]
    fn handles_missing_or_invalid_traceparent_with_fallback() {
        let provider = SdkTracerProvider::builder().build();
        let subscriber = tracing_subscriber::registry()
            .with(tracing_opentelemetry::layer().with_tracer(provider.tracer("test")));
        tracing::subscriber::with_default(subscriber, || {
            let mut event = EventBuilderV10::new()
                .id("test")
                .source("https://example.com")
                .ty("example")
                .build()
                .unwrap();
            assert!(receive_event(&event).is_ok());
            
            event.set_extension("traceparent", "not-a-traceparent");
            assert!(receive_event(&event).is_ok());
            
            event.set_extension("traceparent", 1_i64);
            assert!(receive_event(&event).is_ok());
        });
        provider.shutdown().unwrap();
    }

    #[test]
    fn producer_requires_an_active_trace() {
        assert!(create_event().is_err());
    }

    #[test]
    fn omits_empty_tracestate() {
        let provider = SdkTracerProvider::builder().build();
        let subscriber = tracing_subscriber::registry()
            .with(tracing_opentelemetry::layer().with_tracer(provider.tracer("test")));
        tracing::subscriber::with_default(subscriber, || {
            assert!(create_event().unwrap().extension("tracestate").is_none());
        });
        provider.shutdown().unwrap();
    }
}
