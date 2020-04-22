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
    service_name: String,
    service_version: String,
}

impl ExporterConfig {
    pub fn with_service_name<S: ToString>(self, service_name: S) -> Self {
        ExporterConfig {
            service_name: service_name.to_string(),
            ..self
        }
    }

    pub fn with_service_version<S: ToString>(self, service_version: S) -> Self {
        ExporterConfig {
            service_version: service_version.to_string(),
            ..self
        }
    }

    pub fn build(self) -> Exporter {
        Exporter {
            config: self,
        }
    }
}

impl Default for ExporterConfig {
    fn default() -> Self {
        ExporterConfig {
            service_name: "DEFAULT".to_string(),
            service_version: "0.0.0".to_string(),
        }
    }
}

impl Exporter {
    pub fn builder() -> ExporterConfig {
        ExporterConfig::default()
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
