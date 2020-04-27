use crate::model::span;
use opentelemetry::exporter::trace;
use reqwest::Client;
use std::net::SocketAddr;
use std::time::{Duration};

#[derive(Debug)]
pub struct Uploader {
    client: Client,
    trace_endpoint: String,
}

impl Uploader {
    pub fn new(trace_addr: SocketAddr) -> Self {
        Uploader {
            client: Client::builder()
                .timeout(Duration::from_secs(2))
                .build()
                .expect("Could not build client"),
            trace_endpoint: format!("http://{}/v0.3/traces", trace_addr),
        }
    }

    pub fn upload(&self, batch: span::Batch) -> trace::ExportResult {
        let datadog_msgpack = match rmp_serde::to_vec(&batch) {
            Ok(m) => m,
            Err(_) => return trace::ExportResult::FailedNotRetryable,
        };

        let fut = self.client
            .post(&self.trace_endpoint)
            .header(reqwest::header::CONTENT_TYPE, "application/msgpack")
            .body(datadog_msgpack)
            .send();

        // We spawn in a thread and ignore the result in order to not block the main thread 
        tokio::task::spawn(async move {
            fut.await
        });

        return trace::ExportResult::Success;
    }
}
