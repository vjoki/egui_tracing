use egui::{InnerResponse, RichText, Ui};
use tracing::level_filters::STATIC_MAX_LEVEL;

use super::common::{set_common_props, CommonProps};
use crate::ui::color::{DEBUG_COLOR, ERROR_COLOR, INFO_COLOR, TRACE_COLOR, WARN_COLOR};
use crate::ui::state::LevelFilter;

#[derive(Default)]
pub struct LevelMenuButton<'a> {
    state: Option<&'a mut LevelFilter>,
    common_props: Option<CommonProps>,
}

impl<'a> LevelMenuButton<'a> {
    pub fn state(mut self, v: &'a mut LevelFilter) -> Self {
        self.state = Some(v);
        self
    }

    pub fn show(mut self, ui: &mut Ui) -> bool {
        let state = self.state.as_mut().unwrap();
        let mut changed = false;
        ui.menu_button("Level", |ui| {
            set_common_props(ui, &self.common_props);
            ui.label("Level Filter");

            if STATIC_MAX_LEVEL >= tracing::level_filters::LevelFilter::TRACE {
                changed = changed || ui.add(egui::Checkbox::new(
                    &mut state.trace,
                    RichText::new("TRACE").color(TRACE_COLOR),
                )).changed();
            }

            if STATIC_MAX_LEVEL >= tracing::level_filters::LevelFilter::DEBUG {
                changed = changed || ui.add(egui::Checkbox::new(
                    &mut state.debug,
                    RichText::new("DEBUG").color(DEBUG_COLOR),
                )).changed();
            }

            if STATIC_MAX_LEVEL >= tracing::level_filters::LevelFilter::INFO {
                changed = changed || ui.add(egui::Checkbox::new(
                    &mut state.info,
                    RichText::new("INFO").color(INFO_COLOR),
                )).changed();
            }

            if STATIC_MAX_LEVEL >= tracing::level_filters::LevelFilter::WARN {
                changed = changed || ui.add(egui::Checkbox::new(
                    &mut state.warn,
                    RichText::new("WARN").color(WARN_COLOR),
                )).changed();
            }

            if STATIC_MAX_LEVEL >= tracing::level_filters::LevelFilter::ERROR {
                changed = changed || ui.add(egui::Checkbox::new(
                    &mut state.error,
                    RichText::new("ERROR").color(ERROR_COLOR),
                )).changed();
            }
        });
        changed
    }
}
