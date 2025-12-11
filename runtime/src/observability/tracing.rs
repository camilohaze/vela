//! OpenTelemetry integration for distributed tracing in Vela
//!
//! This module provides distributed tracing capabilities using OpenTelemetry,
//! following W3C Trace Context specification for header propagation.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configuration for the tracing system
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// Service name for this instance
    pub service_name: String,
    /// Service version
    pub service_version: String,
    /// Sampling ratio (0.0 = no sampling, 1.0 = full sampling)
    pub sampling_ratio: f64,
    /// Maximum number of attributes per span
    pub max_attributes: usize,
    /// Maximum number of events per span
    pub max_events: usize,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            service_name: "vela-service".to_string(),
            service_version: "1.0.0".to_string(),
            sampling_ratio: 1.0, // Sample everything in development
            max_attributes: 128,
            max_events: 128,
        }
    }
}

/// Global tracer instance
pub struct Tracer {
    inner: opentelemetry::sdk::trace::Tracer,
    config: TracingConfig,
}

impl Tracer {
    /// Create a new tracer with the given configuration
    pub fn new(config: TracingConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize OpenTelemetry tracer provider
        let tracer_provider = opentelemetry::sdk::trace::TracerProvider::builder()
            .with_config(
                opentelemetry::sdk::trace::config()
                    .with_sampler(opentelemetry::sdk::trace::Sampler::TraceIdRatioBased(
                        config.sampling_ratio,
                    ))
                    .with_max_attributes_per_span(config.max_attributes)
                    .with_max_events_per_span(config.max_events)
                    .with_resource(
                        opentelemetry::sdk::resource::Resource::new(vec![
                            opentelemetry::KeyValue::new("service.name", config.service_name.clone()),
                            opentelemetry::KeyValue::new("service.version", config.service_version.clone()),
                        ])
                    )
            )
            .build();

        let tracer = tracer_provider.tracer("vela-tracer");

        Ok(Self {
            inner: tracer,
            config,
        })
    }

    /// Start a new span with the given name
    pub fn start_span(&self, name: &str) -> Span {
        let inner_span = self.inner.start(name);
        Span {
            inner: inner_span,
            attributes: HashMap::new(),
        }
    }

    /// Start a new span as child of the given parent context
    pub fn start_span_with_parent(&self, name: &str, parent_context: &SpanContext) -> Span {
        let mut span_builder = self.inner.span_builder(name);
        span_builder = span_builder.with_parent_context(parent_context.clone().into());

        let inner_span = span_builder.start(&self.inner);
        Span {
            inner: inner_span,
            attributes: HashMap::new(),
        }
    }
}

/// Represents a span context for propagation
#[derive(Debug, Clone)]
pub struct SpanContext {
    /// Trace ID
    pub trace_id: String,
    /// Span ID
    pub span_id: String,
    /// Trace flags
    pub trace_flags: u8,
}

impl From<opentelemetry::trace::SpanContext> for SpanContext {
    fn from(ctx: opentelemetry::trace::SpanContext) -> Self {
        Self {
            trace_id: ctx.trace_id().to_string(),
            span_id: ctx.span_id().to_string(),
            trace_flags: ctx.trace_flags().to_u8(),
        }
    }
}

impl From<SpanContext> for opentelemetry::trace::SpanContext {
    fn from(ctx: SpanContext) -> Self {
        // This is a simplified conversion - in practice you'd need proper parsing
        opentelemetry::trace::SpanContext::new(
            opentelemetry::trace::TraceId::from_hex(&ctx.trace_id).unwrap_or_default(),
            opentelemetry::trace::SpanId::from_hex(&ctx.span_id).unwrap_or_default(),
            opentelemetry::trace::TraceFlags::new(ctx.trace_flags),
            true,
            opentelemetry::trace::TraceState::default(),
        )
    }
}

/// A span represents a single operation within a trace
pub struct Span {
    inner: opentelemetry::trace::Span,
    attributes: HashMap<String, opentelemetry::Value>,
}

impl Span {
    /// Set an attribute on the span
    pub fn set_attribute(&mut self, key: &str, value: opentelemetry::Value) {
        self.attributes.insert(key.to_string(), value.clone());
        self.inner.set_attribute(opentelemetry::Key::new(key), value);
    }

    /// Add an event to the span
    pub fn add_event(&mut self, name: &str, attributes: Vec<opentelemetry::KeyValue>) {
        self.inner.add_event(name, attributes);
    }

    /// Set the status of the span
    pub fn set_status(&mut self, status: opentelemetry::trace::Status) {
        self.inner.set_status(status);
    }

    /// End the span
    pub fn end(self) {
        self.inner.end();
    }

    /// Get the span context for propagation
    pub fn context(&self) -> SpanContext {
        self.inner.span_context().clone().into()
    }
}

/// Context propagation utilities
pub struct Propagation {
    text_map_propagator: opentelemetry::propagation::TextMapCompositePropagator,
}

impl Propagation {
    /// Create a new propagation instance
    pub fn new() -> Self {
        let text_map_propagator = opentelemetry::propagation::TextMapCompositePropagator::new(vec![
            Box::new(opentelemetry::propagation::TraceContextPropagator::new()),
            Box::new(opentelemetry::propagation::BaggagePropagator::new()),
        ]);

        Self {
            text_map_propagator,
        }
    }

    /// Extract context from headers
    pub fn extract(&self, headers: &HashMap<String, String>) -> Option<SpanContext> {
        let mut extractor = HeaderExtractor(headers.clone());
        let context = self.text_map_propagator.extract(&extractor);

        context.span().span_context().cloned().map(|ctx| ctx.into())
    }

    /// Inject context into headers
    pub fn inject(&self, context: &SpanContext, headers: &mut HashMap<String, String>) {
        let mut injector = HeaderInjector(headers);
        let otel_context = opentelemetry::Context::current_with_span(
            opentelemetry::trace::Span::new(context.clone().into())
        );

        self.text_map_propagator.inject_context(&otel_context, &mut injector);
    }
}

impl Default for Propagation {
    fn default() -> Self {
        Self::new()
    }
}

/// Header extractor for OpenTelemetry propagation
struct HeaderExtractor(HashMap<String, String>);

impl opentelemetry::propagation::Extractor for HeaderExtractor {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|s| s.as_str())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|s| s.as_str()).collect()
    }
}

/// Header injector for OpenTelemetry propagation
struct HeaderInjector<'a>(&'a mut HashMap<String, String>);

impl<'a> opentelemetry::propagation::Injector for HeaderInjector<'a> {
    fn set(&mut self, key: &str, value: opentelemetry::Value) {
        self.0.insert(key.to_string(), value.to_string());
    }
}

/// Global tracing registry
pub struct TracingRegistry {
    tracer: Arc<RwLock<Option<Tracer>>>,
    propagation: Propagation,
}

impl TracingRegistry {
    /// Create a new tracing registry
    pub fn new() -> Self {
        Self {
            tracer: Arc::new(RwLock::new(None)),
            propagation: Propagation::new(),
        }
    }

    /// Initialize the global tracer
    pub async fn init(&self, config: TracingConfig) -> Result<(), Box<dyn std::error::Error>> {
        let tracer = Tracer::new(config)?;
        *self.tracer.write().await = Some(tracer);
        Ok(())
    }

    /// Get the global tracer
    pub async fn get_tracer(&self) -> Option<Tracer> {
        self.tracer.read().await.clone()
    }

    /// Get the propagation utilities
    pub fn propagation(&self) -> &Propagation {
        &self.propagation
    }
}

impl Default for TracingRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global static registry instance
static REGISTRY: once_cell::sync::Lazy<TracingRegistry> = once_cell::sync::Lazy::new(|| {
    TracingRegistry::new()
});

/// Get the global tracing registry
pub fn global_registry() -> &'static TracingRegistry {
    &REGISTRY
}

/// Initialize global tracing
pub async fn init_tracing(config: TracingConfig) -> Result<(), Box<dyn std::error::Error>> {
    global_registry().init(config).await
}

/// Get the global tracer
pub async fn get_tracer() -> Option<Tracer> {
    global_registry().get_tracer().await
}

/// Get the global propagation utilities
pub fn get_propagation() -> &'static Propagation {
    global_registry().propagation()
}