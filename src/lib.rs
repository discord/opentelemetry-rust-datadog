use crate::model::span;
use opentelemetry::api;
use opentelemetry::exporter::trace;
use std::any::Any;
use std::sync::Arc;
use serde_json;

pub mod model;

#[derive(Debug)]
pub struct Exporter {}

impl Default for Exporter {
    fn default() -> Self {
        Exporter {}
    }
}

impl trace::SpanExporter for Exporter {
    fn export(
        &self,
        batch: Vec<Arc<trace::SpanData>>,
    ) -> trace::ExportResult {
        // TODO: What kind of headers matter?
        for span in batch.into_iter() {
            let dd_span = span::Span::from(span.as_ref());
            println!("{}", serde_json::to_string_pretty(&dd_span).unwrap());
        }

        trace::ExportResult::Success
    }

    fn shutdown(&self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }
}
