use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use typed_builder::TypedBuilder;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Batch(pub Vec<PartialTrace>);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PartialTrace(pub Vec<Span>);

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

    /// identifier of the trace
    pub trace_id: u64,
    /// identifier of this span
    pub span_id: u64,
    /// identifier of the span's direct parent
    pub parent_id: u64,
}
