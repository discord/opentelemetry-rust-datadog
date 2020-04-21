use opentelemetry::{
    api::{Key, Provider, Span, TracerGenerics},
    global, sdk,
};
use opentelemetry_rust_datadog::Exporter;

fn main() {
    // Create stdout exporter to be able to retrieve the collected spans.
    let exporter = Exporter::default();

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

    let span_kind_key = Key::new("span.kind");

    global::trace_provider()
        .get_tracer("component-main")
        .with_span("operation", move |span| {
            span.set_attribute(span_kind_key.string("test"));
        });
}
