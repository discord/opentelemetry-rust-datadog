use crate::model::span;
use opentelemetry::exporter::trace;
use reqwest::Client;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::thread::{self, Thread};
use std::time::SystemTime;
use tokio::time::{Instant, Duration};

#[derive(Debug)]
pub struct Uploader {
    client: Client,
    trace_endpoint: String,
}

impl Uploader {
    pub fn new(trace_addr: SocketAddr) -> Self {
        Uploader {
            client: Client::new(),
            trace_endpoint: format!("http://{}/v0.3/traces", trace_addr),
        }
    }

    pub fn upload(&self, batch: span::Batch) -> trace::ExportResult {
        let datadog_json = match serde_json::to_string(&batch) {
            Ok(json) => json,
            Err(_) => return trace::ExportResult::FailedNotRetryable,
        };

        let fut = self.client
            .post(&self.trace_endpoint)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(datadog_json)
            .send();

        tokio::task::spawn(async move {
            fut.await
        });

        // let resp = timeout(task, Some(Duration::from_secs(5)));

        // println!("{:#?}: {:?}", SystemTime::now(), resp);

        // if let Ok(resp) = resp {
        //     if let Ok(resp) = resp {
        //         if resp.status().is_success() {
        //             return trace::ExportResult::Success;
        //         }
        //     }
        // }

        return trace::ExportResult::Success;
    }
}

// pub(crate) fn timeout<F, I, E>(fut: F, timeout: Option<Duration>) -> Result<I, Waited<E>>
// where
//     F: Future<Output = Result<I, E>>,
// {
//     let deadline = timeout.map(|d| {
//         // log::trace!("wait at most {:?}", d);
//         Instant::now() + d
//     });

//     let thread = ThreadWaker(thread::current());
//     // Arc shouldn't be necessary, since `Thread` is reference counted internally,
//     // but let's just stay safe for now.
//     let waker = futures_util::task::waker(Arc::new(thread));
//     let mut cx = Context::from_waker(&waker);

//     futures_util::pin_mut!(fut);

//     loop {
//         match fut.as_mut().poll(&mut cx) {
//             Poll::Ready(Ok(val)) => return Ok(val),
//             Poll::Ready(Err(err)) => return Err(Waited::Inner(err)),
//             Poll::Pending => (), // fallthrough
//         }

//         if let Some(deadline) = deadline {
//             let now = Instant::now();
//             if now >= deadline {
//                 // log::trace!("wait timeout exceeded");
//                 return Err(Waited::TimedOut);
//             }

//             // log::trace!("({:?}) park timeout {:?}", thread::current().id(), deadline - now);
//             // thread::park_timeout(deadline - now);
//         } else {
//             // log::trace!("({:?}) park without timeout", thread::current().id());
//             // thread::park();
//         }
//     }
// }

// #[derive(Debug)]
// pub(crate) enum Waited<E> {
//     TimedOut,
//     Inner(E),
// }

// struct ThreadWaker(Thread);

// impl futures_util::task::ArcWake for ThreadWaker {
//     fn wake_by_ref(arc_self: &Arc<Self>) {
//         arc_self.0.unpark();
//     }
// }
