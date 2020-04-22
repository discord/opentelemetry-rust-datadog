#[macro_use]
extern crate tracing;

use opentelemetry::{api::Provider, global, sdk};
use opentelemetry_datadog::{Exporter};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

fn main() {
    // Create datadog exporter to be able to retrieve the collected spans.
    let exporter = Exporter::builder().build();

    // For the demonstration, use `Sampler::Always` sampler to sample all traces. In a production
    // application, use `Sampler::Parent` or `Sampler::Probability` with a desired probability.
    let provider = sdk::Provider::builder()
        .with_simple_exporter(exporter)
        .with_config(sdk::Config {
            default_sampler: Box::new(sdk::Sampler::Always),
            ..Default::default()
        })
        .build();
    global::set_provider(provider);

    // Create a new tracer
    let tracer = global::trace_provider().get_tracer("component_name");

    // Create a new OpenTelemetry tracing layer
    let telemetry = OpenTelemetryLayer::with_tracer(tracer);

    let subscriber = Registry::default().with(telemetry);

    // Trace executed code
    tracing::subscriber::with_default(subscriber, || {
        let root = span!(tracing::Level::TRACE, "app_start", work_units = 2);
        let _enter = root.enter();

        error!("This event will be logged in the root span.");
    });
}
