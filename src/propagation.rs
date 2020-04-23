use opentelemetry::api;	

const HTTP_HEADER_TRACE_ID: &'static str = "x-datadog-trace-id";		
const HTTP_HEADER_PARENT_ID: &'static str = "x-datadog-parent-id";		
const HTTP_HEADER_SAMPLING_PRIORITY: &'static str = "x-datadog-sampling-priority";		
// TODO: Implement origin propagation once the Context API is stable
// const HTTP_HEADER_ORIGIN: &'static str = "x-datadog-origin";

enum SamplingPriority {
    UserReject = -1,
    AutoReject = 0,
    AutoKeep = 1,
    UserKeep = 2,
}

enum ExtractError {
    InvalidTraceId,
    InvalidSpanId,
    InvalidSamplingPriority,
    InvalidSpanContext,
}

pub struct DatadogPropagator {}

impl DatadogPropagator {
    fn extract_trace_id(&self, trace_id: &str) -> Result<api::TraceId, ExtractError> {
        u64::from_str_radix(trace_id, 10)
            .map(|id| api::TraceId::from_u128(id as u128))
            .map_err(|_| ExtractError::InvalidTraceId)
    }

    fn extract_span_id(&self, span_id: &str) -> Result<api::SpanId, ExtractError> {
        u64::from_str_radix(span_id, 10)
            .map(api::SpanId::from_u64)
            .map_err(|_| ExtractError::InvalidSpanId)
    }

    fn extract_sampling_priority(&self, sampling_priority: &str) -> Result<SamplingPriority, ExtractError> {
        let i = i32::from_str_radix(sampling_priority, 10)
            .map_err(|_| ExtractError::InvalidSamplingPriority)?;
        
        match i {
            -1 => Ok(SamplingPriority::UserReject),
            0 => Ok(SamplingPriority::AutoReject),
            1 => Ok(SamplingPriority::AutoKeep),
            2 => Ok(SamplingPriority::UserKeep),
            _ => Err(ExtractError::InvalidSamplingPriority),
        }
    }

    fn extract(&self, carrier: &dyn api::Carrier) -> Result<api::SpanContext, ExtractError> {
        let trace_id = self
            .extract_trace_id(carrier.get(HTTP_HEADER_TRACE_ID).unwrap_or(""))?;
        let span_id = self
            .extract_span_id(carrier.get(HTTP_HEADER_PARENT_ID).unwrap_or(""))?;
        let sampling_priority = self
            .extract_sampling_priority(carrier.get(HTTP_HEADER_SAMPLING_PRIORITY).unwrap_or(""))?;
        let sampled = match sampling_priority {
            SamplingPriority::UserReject | SamplingPriority::AutoReject => 0,
            SamplingPriority::UserKeep | SamplingPriority::AutoKeep => api::TRACE_FLAG_SAMPLED,
        };

        let span_context = api::SpanContext::new(trace_id, span_id, sampled, true);
        if span_context.is_valid() {
            Ok(span_context)
        } else {
            Err(ExtractError::InvalidSpanContext)
        }
    }
}

impl api::HttpTextFormat for DatadogPropagator {
    fn inject(&self, context: api::SpanContext, carrier: &mut dyn api::Carrier) {
        if context.is_valid() {
            carrier.set(
                HTTP_HEADER_TRACE_ID,
                (context.trace_id().to_u128() as u64).to_string(),
            );
            carrier.set(
                HTTP_HEADER_PARENT_ID,
                context.span_id().to_u64().to_string(),
            );

            let sampling_priority = if context.is_sampled() { SamplingPriority::AutoKeep } else { SamplingPriority::AutoReject };

            carrier.set(
                HTTP_HEADER_SAMPLING_PRIORITY,
                (sampling_priority as i32).to_string()
            );
        }
    }

    fn extract(&self, carrier: &dyn api::Carrier) -> api::SpanContext {
        self.extract(carrier).unwrap_or_else(|_| api::SpanContext::empty_context())
    }
}

#[cfg(test)]
mod tests {
    // TODO
}
