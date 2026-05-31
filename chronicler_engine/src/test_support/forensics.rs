//! Forensics collector for test failure diagnostics.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use parking_lot::Mutex;
use serde::Serialize;
use tracing::{span, Event, Level, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

/// Redacted marker for sensitive fields.
const REDACTED: &str = "[REDACTED]";

/// Field names that should be redacted.
const SENSITIVE_FIELDS: &[&str] = &["api_key", "prompt", "raw_response", "authorization"];

/// Buffered span data for serialization.
#[derive(Serialize, Clone)]
pub struct SpanData {
    pub name: String,
    pub id: u64,
    pub parent_id: Option<u64>,
    pub fields: HashMap<String, String>,
    pub entered_at: Option<u64>,
    pub exited_at: Option<u64>,
}

/// Buffered event data for serialization.
#[derive(Serialize, Clone)]
pub struct EventData {
    pub message: String,
    pub level: String,
    pub span_id: Option<u64>,
    pub fields: HashMap<String, String>,
    pub timestamp: u64,
}

/// Complete forensics snapshot for a test.
#[derive(Serialize)]
pub struct ForensicsSnapshot {
    pub test_name: String,
    pub timestamp: String,
    pub spans: Vec<SpanData>,
    pub events: Vec<EventData>,
    pub duration_ms: u64,
}

/// Forensics collector that buffers tracing data.
pub struct ForensicsCollector {
    spans: Arc<Mutex<Vec<SpanData>>>,
    events: Arc<Mutex<Vec<EventData>>>,
    start_time: Instant,
    test_name: Arc<Mutex<String>>,
}

impl ForensicsCollector {
    /// Create a new forensics collector.
    pub fn new() -> Self {
        Self {
            spans: Arc::new(Mutex::new(Vec::new())),
            events: Arc::new(Mutex::new(Vec::new())),
            start_time: Instant::now(),
            test_name: Arc::new(Mutex::new(String::new())),
        }
    }

    /// Set the test name for this forensics capture.
    pub fn set_test_name(&self, name: &str) {
        let mut test_name = self.test_name.lock();
        *test_name = name.to_string();
    }

    /// Capture forensics data to a JSON file on test failure.
    pub fn capture_on_failure(&self) -> std::io::Result<PathBuf> {
        let test_name = self.test_name.lock().clone();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Create diagnostics directory
        let diag_dir = PathBuf::from("tmp/diagnostics");
        fs::create_dir_all(&diag_dir)?;

        // Build snapshot
        let snapshot = ForensicsSnapshot {
            test_name,
            timestamp,
            spans: self.spans.lock().clone(),
            events: self.events.lock().clone(),
            duration_ms: self.start_time.elapsed().as_millis() as u64,
        };

        // Write JSON file
        let filename = format!(
            "forensics_{}_{}.json",
            snapshot.test_name.replace("::", "_"),
            chrono::Utc::now().timestamp()
        );
        let path = diag_dir.join(&filename);

        let json = serde_json::to_string_pretty(&snapshot)?;
        fs::write(&path, json)?;

        Ok(path)
    }

    /// Redact sensitive field values.
    fn redact_field(key: &str, value: &str) -> String {
        if SENSITIVE_FIELDS.contains(&key.to_lowercase().as_str()) {
            REDACTED.to_string()
        } else if value.len() > 10000 {
            // Truncate very long strings
            format!("{}... [truncated]", &value[..10000])
        } else {
            value.to_string()
        }
    }
}

impl Default for ForensicsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Layer implementation for the forensics collector.
pub struct ForensicsLayer {
    inner: Arc<ForensicsCollector>,
}

impl ForensicsLayer {
    pub fn new(collector: Arc<ForensicsCollector>) -> Self {
        Self { inner: collector }
    }
}

impl<S: Subscriber> Layer<S> for ForensicsLayer {
    fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, _ctx: Context<'_, S>) {
        let mut span_data = SpanData {
            name: attrs.metadata().name().to_string(),
            id: id.into_u64(),
            parent_id: None,
            fields: HashMap::new(),
            entered_at: None,
            exited_at: None,
        };

        // Record initial fields
        attrs.record(&mut SpanRecorder {
            fields: &mut span_data.fields,
        });

        self.inner.spans.lock().push(span_data);
    }

    fn on_record(&self, id: &span::Id, values: &span::Record<'_>, _ctx: Context<'_, S>) {
        let mut spans = self.inner.spans.lock();
        if let Some(span) = spans.iter_mut().find(|s| s.id == id.into_u64()) {
            values.record(&mut SpanRecorder {
                fields: &mut span.fields,
            });
        }
    }

    fn on_enter(&self, id: &span::Id, _ctx: Context<'_, S>) {
        let mut spans = self.inner.spans.lock();
        if let Some(span) = spans.iter_mut().find(|s| s.id == id.into_u64()) {
            span.entered_at = Some(span.id);
        }
    }

    fn on_exit(&self, id: &span::Id, _ctx: Context<'_, S>) {
        let mut spans = self.inner.spans.lock();
        if let Some(span) = spans.iter_mut().find(|s| s.id == id.into_u64()) {
            span.exited_at = Some(span.id);
        }
    }

    fn on_close(&self, _id: span::Id, _ctx: Context<'_, S>) {
        // Span closed, no action needed
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let mut fields = HashMap::new();
        event.record(&mut SpanRecorder {
            fields: &mut fields,
        });

        let metadata = event.metadata();
        let level_str = match *metadata.level() {
            Level::ERROR => "error",
            Level::WARN => "warn",
            Level::INFO => "info",
            Level::DEBUG => "debug",
            Level::TRACE => "trace",
        };

        let event_data = EventData {
            message: metadata.name().to_string(),
            level: level_str.to_string(),
            span_id: ctx.current_span().id().map(|id| id.into_u64()),
            fields,
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        self.inner.events.lock().push(event_data);
    }
}

/// Helper to record field values.
struct SpanRecorder<'a> {
    fields: &'a mut HashMap<String, String>,
}

impl tracing::field::Visit for SpanRecorder<'_> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        let value_str = format!("{value:?}");
        let redacted = ForensicsCollector::redact_field(field.name(), &value_str);
        self.fields.insert(field.name().to_string(), redacted);
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        let redacted = ForensicsCollector::redact_field(field.name(), value);
        self.fields.insert(field.name().to_string(), redacted);
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }
}
