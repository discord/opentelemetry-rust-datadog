use crate::model::span;
use opentelemetry::api;
use opentelemetry::exporter::trace;
use std::any::Any;
use std::sync::Arc;
use serde_json;

pub mod model;

#[derive(Debug)]
pub struct Exporter {
    config: ExporterConfig,
}

#[derive(Clone, Debug)]
pub struct ExporterConfig {
    pub service_name: String
}

impl Default for ExporterConfig {
    fn default() -> Self {
        ExporterConfig {
            service_name: "DEFAULT".to_string(),
        }
    }
}

impl Exporter {
    pub fn from_config(config: ExporterConfig) -> Self {
        Exporter {
            config
        }
    }
}

impl trace::SpanExporter for Exporter {
    fn export(
        &self,
        batch: Vec<Arc<trace::SpanData>>,
    ) -> trace::ExportResult {
        // TODO: What kind of headers matter?
        // TODO: How to sampling?

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
