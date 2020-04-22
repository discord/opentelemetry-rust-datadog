use opentelemetry::api;
use opentelemetry::exporter::trace;
use opentelemetry::sdk::trace::evicted_hash_map::EvictedHashMap;
use serde::{Serialize, Deserialize};
use serde_json;
use std::collections::HashMap;
use std::time;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder, Serialize, Deserialize, Debug, Clone)]
pub struct Span {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// operation name
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// service name (i.e. "grpc.server", "http.request")
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// resource name (i.e. "/user?id=123", "SELECT * FROM users")
    pub resource: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    /// protocol associated with the span (i.e. "web", "db", "cache")
    pub span_type: Option<String>,

    // tags / metadata
    #[serde(default)]
    /// arbitrary map of metdata
    pub meta: HashMap<String, String>,
    /// error status of the span; 0 means no errors
    pub error: i64,
    #[serde(default)]
    /// arbitrary map of numeric metrics
    pub metrics: HashMap<String, f64>,

    /// span start time in nanoseconds since epoch
    pub start: i64,
    /// duration of the span in nanoseconds
    pub duration: i64,

    /// identifier of the root span
    /// FIXME: I think datadog expects this to be the spanid of the root span, not a general trace id
    ///        and it expects u64
    pub trace_id: u128,
    /// identifier of this span
    pub span_id: u64,
    /// identifier of the span's direct parent
    pub parent_id: u64,
}

fn duration_to_ns(r: Result<time::Duration, time::SystemTimeError>) -> i64 {
    match r {
        Ok(d) => d.as_nanos().min(std::i64::MAX as u128) as i64,
        Err(e) => -(e.duration().as_nanos().min(std::i64::MAX as u128) as i64)
    }
}

fn split_attributes(attributes: &EvictedHashMap) -> (HashMap<String, String>, HashMap<String, f64>) {
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

impl From<&trace::SpanData> for Span {
    fn from(span_data: &trace::SpanData) -> Self {
        let (mut meta, metrics) = split_attributes(&span_data.attributes);
        let name = meta
            .get("span.name")
            .map(String::clone)
            .or_else(|| Some(span_data.name.clone()));
        let service = meta.get("service.name").map(String::clone);
        let resource = meta.get("resource.name").map(String::clone);
        let span_type = meta.get("span.type").map(String::clone);
        let start = duration_to_ns(span_data.start_time.duration_since(time::SystemTime::UNIX_EPOCH));
        let duration = duration_to_ns(span_data.end_time.duration_since(span_data.start_time));

        let mut error = 0;

        match &span_data.status_code {
            api::StatusCode::OK => (),
            sc => {
                error = 1;
                meta.insert("status_code".to_string(), format!("{:?}", sc));
            },
        }

        let span_kind = match span_data.span_kind {
            api::SpanKind::Client => "client",
            api::SpanKind::Server => "server",
            api::SpanKind::Producer => "producer",
            api::SpanKind::Consumer => "consumer",
            api::SpanKind::Internal => "internal",
        };

        meta.insert("span.kind".to_string(), span_kind.to_string());

        // TODO: Include the follow span_data data
        //       * status_message
        //       * trace_flags
        //       * is_remote

        Span::builder()
            .name(name)
            .service(service)
            .resource(resource)
            .span_type(span_type)
            .meta(meta)
            .error(error)
            .metrics(metrics)
            .start(start)
            .duration(duration)
            .trace_id(span_data.context.trace_id().to_u128())
            .span_id(span_data.context.span_id().to_u64())
            .parent_id(span_data.parent_span_id.to_u64())
            .build()
    }
}
