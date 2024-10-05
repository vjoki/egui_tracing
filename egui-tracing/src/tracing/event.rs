use std::collections::BTreeMap;
use std::fmt::Debug;

use chrono::{DateTime, Local};
use tracing::field::{Field, Visit};
use tracing::{Event, Metadata};

#[derive(Debug, Clone)]
pub struct CollectedEvent {
    pub target: String,
    pub level: tracing::Level,
    pub fields: BTreeMap<&'static str, String>,
    pub time: DateTime<Local>,
}

impl CollectedEvent {
    pub fn new(fields: BTreeMap<&'static str, String>, meta: &Metadata) -> Self {
        CollectedEvent {
            level: meta.level().to_owned(),
            time: Local::now(),
            target: meta.target().to_owned(),
            fields,
        }
    }
}
