use core::hash;

use globset::{Glob, GlobSet};
use serde::{Deserialize, Serialize};
use tracing::{level_filters::STATIC_MAX_LEVEL, Level};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LogsState {
    pub level_filter: LevelFilter,
    pub target_filter: TargetFilter,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LevelFilter {
    pub trace: bool,
    pub debug: bool,
    pub info: bool,
    pub warn: bool,
    pub error: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TargetFilter {
    pub input: String,
    pub targets: Vec<Glob>,
    #[serde(skip)]
    pub glob: Option<GlobSet>,
}

impl hash::Hash for TargetFilter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.input.hash(state);
        self.targets.hash(state);
    }
}

impl Default for LevelFilter {
    fn default() -> Self {
        Self {
            trace: false,
            debug: false,
            info: STATIC_MAX_LEVEL >= tracing::level_filters::LevelFilter::INFO,
            warn: STATIC_MAX_LEVEL >= tracing::level_filters::LevelFilter::WARN,
            error: STATIC_MAX_LEVEL >= tracing::level_filters::LevelFilter::ERROR,
        }
    }
}

impl LevelFilter {
    pub fn get(&self, level: Level) -> bool {
        match level {
            Level::TRACE => self.trace,
            Level::DEBUG => self.debug,
            Level::INFO => self.info,
            Level::WARN => self.warn,
            Level::ERROR => self.error,
        }
    }

    pub fn max_level(&self) -> Option<Level> {
        if self.trace {
            Some(Level::TRACE)
        } else if self.debug {
            Some(Level::DEBUG)
        } else if self.info {
            Some(Level::INFO)
        } else if self.warn {
            Some(Level::WARN)
        } else if self.error {
            Some(Level::ERROR)
        } else { None }
    }
}
