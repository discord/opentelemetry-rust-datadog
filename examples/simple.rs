use opentelemetry::{
    api::{self, Provider, Span, TracerGenerics},
    global, sdk,
};
use opentelemetry_datadog::Exporter;
use tokio;

#[tokio::main]
async fn main() {
    // Create datadog exporter to be able to retrieve the collected spans.
    let exporter = Exporter::builder()
        .with_trace_addr("127.0.0.1:3022".parse().unwrap())
        .build();

    // Batching is required to offload from the main thread
    let batch =
        sdk::BatchSpanProcessor::builder(exporter, tokio::spawn, tokio::time::interval).build();

    // For the demonstration, use `Sampler::Always` sampler to sample all traces. In a production
    // application, use `Sampler::Parent` or `Sampler::Probability` with a desired probability.
    let provider = sdk::Provider::builder()
        .with_batch_exporter(batch)
        .with_config(sdk::Config {
            default_sampler: Box::new(sdk::Sampler::Always),
            ..Default::default()
        })
        .build();
    global::set_provider(provider);

    global::trace_provider()
        .get_tracer("component-main")
        .with_span("operation", move |_span| {
            println!("in operation");
            global::trace_provider()
                .get_tracer("component-main")
                .with_span("nested", move | _span| {
                    println!("in nested");

                })
        });

    global::trace_provider()
        .get_tracer("component-main")
        .with_span("error_operation", move |span| {
            println!("in error_operation");
            span.set_status(api::StatusCode::Internal, "Oops.".to_string())
        });

    // We must delay here until https://github.com/open-telemetry/opentelemetry-rust/issues/90 is fixed
    tokio::time::delay_for(tokio::time::Duration::from_secs(10)).await;

}
