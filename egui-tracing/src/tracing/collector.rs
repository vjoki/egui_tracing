use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;

use egui::mutex::{RwLock, RwLockReadGuard};
use tracing::field::{Field, Visit};
use tracing::level_filters::LevelFilter;
use tracing::{span, Event, Level, Subscriber};
#[cfg(feature = "log")]
use tracing_log::NormalizeEvent;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

use super::event::CollectedEvent;
use super::level_filter::{AtomicLevelFilter, DynamicLevelFilter};

#[derive(Clone)]
pub struct EventCollector {
    events: Arc<RwLock<VecDeque<CollectedEvent>>>,
    max_events: usize,
    level: Arc<AtomicLevelFilter>,
}

impl EventCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_level(self, level: Option<Level>) -> Self {
        Self { level: Arc::from(AtomicLevelFilter::from(LevelFilter::from(level))), ..self }
    }

    pub fn max_log_entries(self, max_events: usize) -> Self {
        Self { max_events, ..self }
    }

    pub fn with_level_filter<S: Subscriber + for<'a> LookupSpan<'a>>(self) -> tracing_subscriber::filter::Filtered<Self, DynamicLevelFilter, S> {
        let level = self.level.clone();
        self.with_filter(DynamicLevelFilter::new(level))
    }

    /// Use with care, will deadlock if held carelessly.
    pub fn events<'a, 'b: 'a>(&'b self) -> RwLockReadGuard<'a, VecDeque<CollectedEvent>> {
        self.events.read()
    }

    /// Set max filter level.
    pub fn set_max_filter_level(&self, level: Option<Level>) {
        if self.level.set_from_level(level) {
            tracing_core::callsite::rebuild_interest_cache();
        }
    }

    pub fn clear(&self) {
        let mut events = self.events.write();
        events.clear();
        events.shrink_to_fit();
    }
}

impl Default for EventCollector {
    fn default() -> Self {
        Self {
            events: Default::default(),
            level: Arc::from(AtomicLevelFilter::from(LevelFilter::INFO)),
            max_events: 10000,
        }
    }
}

impl<S> Layer<S> for EventCollector
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn max_level_hint(&self) -> Option<LevelFilter> {
        Some(self.level.as_level_filter())
    }

    fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(id) {
            attrs.metadata().level();
            // Collect any fields that have been assigned during span creation.
            let mut fields = BTreeMap::new();
            attrs.record(&mut FieldVisitor(&mut fields));
            span.extensions_mut().insert(SpanFieldStorage(fields));
        }
    }

    fn on_record(&self, id: &span::Id, values: &span::Record<'_>, ctx: Context<'_, S>) {
        // Collect any span field assignments that occur after span creation.
        if let Some(span) = ctx.span(id) {
            if let Some(storage) = span.extensions_mut().get_mut::<SpanFieldStorage>() {
                values.record(&mut FieldVisitor(&mut storage.0));
            }
        }
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        #[cfg(feature = "log")]
        let normalized_meta = event.normalized_metadata();
        #[cfg(feature = "log")]
        let meta = normalized_meta.as_ref().unwrap_or_else(|| event.metadata());
        #[cfg(not(feature = "log"))]
        let meta = event.metadata();

        let mut fields: BTreeMap<&'static str, String> = BTreeMap::new();
        if let Some(scope) = ctx.event_scope(event) {
            for s in scope.from_root() {
                if let Some(storage) = s.extensions().get::<SpanFieldStorage>() {
                    fields.extend(storage.0.iter().map(|(k, v)| {
                        let k: &'static str = *k;
                        (k, v.clone())
                    }));
                }
            }
        }
        event.record(&mut FieldVisitor(&mut fields));

        let mut events = self.events.write();
        if events.len() >= self.max_events {
            events.pop_front();
        }
        events.push_back(CollectedEvent::new(fields, meta));
    }
}

struct SpanFieldStorage(BTreeMap<&'static str, String>);

struct FieldVisitor<'a>(&'a mut BTreeMap<&'static str, String>);

impl<'a> Visit for FieldVisitor<'a> {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.0.entry(field.name()).or_insert_with(|| format!("{:?}", value));
    }
}
