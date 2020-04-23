use crate::model::span;
use opentelemetry::api;
use opentelemetry::exporter::trace;
use opentelemetry::sdk::trace::evicted_hash_map::EvictedHashMap;
use std::any::Any;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time;
use tracing::{info, trace, error};

pub mod model;
pub mod propagation;
pub(crate) mod uploader;

#[derive(Debug)]
pub struct Exporter {
    config: ExporterConfig,
    uploader: uploader::Uploader,

}

#[derive(Clone, Debug)]
pub struct ExporterConfig {
    service_name: String,
    service_version: String,
    trace_addr: SocketAddr,
    global_tags: HashMap<String, String>,
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

    pub fn with_trace_addr(self, trace_addr: SocketAddr) -> Self {
        ExporterConfig { trace_addr, ..self }
    }

    pub fn with_global_tags(self, global_tags: HashMap<String, String>) -> Self {
        ExporterConfig {
            global_tags,
            ..self
        }
    }

    pub fn build(self) -> Exporter {
        let uploader = uploader::Uploader::new(
                self.trace_addr.clone(),
            );
        Exporter {
            config: self,
            uploader,
        }
    }
}

impl Default for ExporterConfig {
    fn default() -> Self {
        ExporterConfig {
            service_name: "DEFAULT".to_string(),
            service_version: "0.0.0".to_string(),
            trace_addr: "127.0.0.1:8126".parse().unwrap(),
            global_tags: HashMap::new(),
        }
    }
}

fn duration_to_ns(r: Result<time::Duration, time::SystemTimeError>) -> i64 {
    match r {
        Ok(d) => d.as_nanos().min(std::i64::MAX as u128) as i64,
        Err(e) => -(e.duration().as_nanos().min(std::i64::MAX as u128) as i64),
    }
}

fn split_attributes(
    attributes: &EvictedHashMap,
) -> (HashMap<String, String>, HashMap<String, f64>) {
    let mut meta = HashMap::new();
    let mut metrics = HashMap::new();

    for (k, v) in attributes.iter() {
        let metric_value = match v {
            api::Value::Bool(b) => Some(*b as i64 as f64),
            api::Value::I64(i) => Some(*i as f64),
            api::Value::U64(u) => Some(*u as f64),
            api::Value::F64(f) => Some(*f),
            _ => None,
        };

        if let Some(metric_value) = metric_value {
            metrics.insert(k.inner().to_string(), metric_value);
        }

        meta.insert(k.inner().to_string(), v.to_string());
    }

    (meta, metrics)
}

impl Exporter {
    pub fn builder() -> ExporterConfig {
        ExporterConfig::default()
    }

    fn convert_span(&self, s: &trace::SpanData) -> span::Span {
        let (mut meta, metrics) = split_attributes(&s.attributes);
        let name = meta
            .get("span.name")
            .map(String::clone)
            .or_else(|| Some(s.name.clone()));
        let service = Some(meta
            .get("service.name")
            .unwrap_or(&self.config.service_name)
            .clone());
        let resource = meta.get("resource.name").map(String::clone);
        let span_type = meta.get("span.type").map(String::clone);
        let start = duration_to_ns(s.start_time.duration_since(time::SystemTime::UNIX_EPOCH));
        let duration = duration_to_ns(s.end_time.duration_since(s.start_time));
        let trace_id = s.context.trace_id().to_u128() as u64;

        let mut error = 0;

        match &s.status_code {
            api::StatusCode::OK => (),
            sc => {
                error = 1;
                meta.insert("error.type".to_string(), format!("StatusCode::{:?}", sc));
                meta.insert("error.msg".to_string(), s.status_message.clone());
            }
        }

        let span_kind = match s.span_kind {
            api::SpanKind::Client => "client",
            api::SpanKind::Server => "server",
            api::SpanKind::Producer => "producer",
            api::SpanKind::Consumer => "consumer",
            api::SpanKind::Internal => "internal",
        };

        meta.insert("span.kind".to_string(), span_kind.to_string());
        meta.insert("service_version".to_string(), self.config.service_version.clone());

        span::Span::builder()
            .name(name)
            .service(service)
            .resource(resource)
            .span_type(span_type)
            .meta(meta)
            .error(error)
            .metrics(metrics)
            .start(start)
            .duration(duration)
            .trace_id(trace_id)
            .span_id(s.context.span_id().to_u64())
            .parent_id(s.parent_span_id.to_u64())
            .build()
    }
}

impl trace::SpanExporter for Exporter {
    fn export(&self, batch: Vec<Arc<trace::SpanData>>) -> trace::ExportResult {
        trace!("exporting batch");
        // TODO: What kind of headers matter?
        // TODO: How to sampling?

        // Group into partial traces
        let mut partial_traces = HashMap::new();

        for span in batch.into_iter() {
            let dd_span = self.convert_span(span.as_ref());
            let partial_trace = partial_traces.entry(dd_span.trace_id).or_insert_with(|| Vec::new());
            partial_trace.push(dd_span);
        }

        let mut dd_batch = span::Batch(Vec::new());

        for (_, partial_trace) in partial_traces.into_iter() {
            dd_batch.0.push(span::PartialTrace(partial_trace));
        }

        let res = self.uploader.upload(dd_batch);
        match res {
            trace::ExportResult::Success => (),
            _ => {
                error!("export failed: {:?}", res);
            },
        };
        res
    }

    fn shutdown(&self) {
        info!("shutting down");
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
