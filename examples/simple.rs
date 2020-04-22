use opentelemetry::{
    api::{ Provider, TracerGenerics},
    global, sdk,
};
use opentelemetry_rust_datadog::{Exporter, ExporterConfig};

fn main() {
    // Create datadog exporter to be able to retrieve the collected spans.
    let exporter = Exporter::from_config(ExporterConfig::default());

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

    global::trace_provider()
        .get_tracer("component-main")
        .with_span("operation", move |_span| {});
}
