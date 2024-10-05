use std::sync::atomic::{self, AtomicU8};
use std::sync::Arc;
use tracing::level_filters::LevelFilter;
use tracing::{Level, Metadata, Subscriber};
use tracing_core::Interest;
use tracing_subscriber::layer::{Context, Filter};
use tracing_subscriber::registry::LookupSpan;

pub struct DynamicLevelFilter(Arc<AtomicLevelFilter>);

impl DynamicLevelFilter {
    pub(super) fn new(filter: Arc<AtomicLevelFilter>) -> Self {
        Self(filter)
    }
}

impl<S> Filter<S> for DynamicLevelFilter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn enabled(&self, meta: &Metadata<'_>, _: &Context<'_, S>) -> bool {
        self.0.is_level_enabled(meta.level())
    }

    fn callsite_enabled(&self, meta: &'static Metadata<'static>) -> Interest {
        if self.0.is_level_enabled(meta.level()) {
            Interest::always()
        } else {
            Interest::never()
        }
    }

    fn max_level_hint(&self) -> Option<LevelFilter> {
        Some(self.0.as_level_filter())
    }
}

pub(super) struct AtomicLevelFilter(AtomicU8);

impl AtomicLevelFilter {
    pub(super) fn as_level_filter(&self) -> LevelFilter {
        let v = self.0.load(atomic::Ordering::Acquire);
        match v {
            0 => LevelFilter::OFF,
            1 => LevelFilter::ERROR,
            2 => LevelFilter::WARN,
            3 => LevelFilter::INFO,
            4 => LevelFilter::DEBUG,
            5 => LevelFilter::TRACE,
            _ => unreachable!()
        }
    }

    pub(super) fn is_level_enabled(&self, level: &Level) -> bool {
        let v = self.0.load(atomic::Ordering::Acquire);
        match v {
            0 => false,
            1 => Level::ERROR >= *level,
            2 => Level::WARN >= *level,
            3 => Level::INFO >= *level,
            4 => Level::DEBUG >= *level,
            5 => Level::TRACE >= *level,
            _ => unreachable!()
        }
    }

    pub(super) fn set_from_level(&self, value: Option<Level>) -> bool {
        let v = match value {
            None => 0,
            Some(Level::ERROR) => 1,
            Some(Level::WARN) => 2,
            Some(Level::INFO) => 3,
            Some(Level::DEBUG) => 4,
            Some(Level::TRACE) => 5,
        };
        let prev = self.0.swap(v, atomic::Ordering::Release);
        prev != v
    }
}

impl From<LevelFilter> for AtomicLevelFilter {
    fn from(value: LevelFilter) -> Self {
        Self::from(value.into_level())
    }
}

impl From<Option<Level>> for AtomicLevelFilter {
    fn from(value: Option<Level>) -> Self {
        let v = match value {
            None => 0,
            Some(Level::ERROR) => 1,
            Some(Level::WARN) => 2,
            Some(Level::INFO) => 3,
            Some(Level::DEBUG) => 4,
            Some(Level::TRACE) => 5,
        };
        Self(AtomicU8::new(v))
    }
}
