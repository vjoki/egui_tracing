use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;

use egui::mutex::{RwLock, RwLockReadGuard};
use tracing::field::{Field, Visit};
use tracing::{span, Event, Level, Subscriber};
#[cfg(feature = "log")]
use tracing_log::NormalizeEvent;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

use super::event::CollectedEvent;

#[derive(Clone, Debug)]
pub enum AllowedTargets {
    All,
    Selected(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct EventCollector {
    allowed_targets: AllowedTargets,
    level: Level,
    events: Arc<RwLock<VecDeque<CollectedEvent>>>,
    max_events: usize,
}

impl EventCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_level(self, level: Level) -> Self {
        Self { level, ..self }
    }

    pub fn allowed_targets(self, allowed_targets: AllowedTargets) -> Self {
        Self {
            allowed_targets,
            ..self
        }
    }

    pub fn max_log_entries(self, max_events: usize) -> Self {
        Self { max_events, ..self }
    }

    /// Use with care, will deadlock if held carelessly.
    pub fn events<'a, 'b: 'a>(&'b self) -> RwLockReadGuard<'a, VecDeque<CollectedEvent>> {
        self.events.read()
    }

    fn collect(&self, event: CollectedEvent) {
        if event.level <= self.level {
            let should_collect = match self.allowed_targets {
                AllowedTargets::All => true,
                AllowedTargets::Selected(ref selection) => selection
                    .iter()
                    .any(|target| event.target.starts_with(target)),
            };
            if should_collect {
                self.events.lock().unwrap().push(event);
            }
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
            allowed_targets: AllowedTargets::All,
            level: Level::TRACE, // capture everything by default.
            events: Default::default(),
            max_events: 10000,
        }
    }
}

impl<S> Layer<S> for EventCollector
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
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
