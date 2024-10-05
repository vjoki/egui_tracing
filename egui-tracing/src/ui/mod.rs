mod color;
mod components;
mod state;

use std::cell::RefCell;

use egui::{Label, Response, TextStyle, TextWrapMode};
use globset::GlobSetBuilder;

use self::color::ToColor32;
use self::components::common::CommonProps;
use self::components::constants;
use self::components::level_menu_button::LevelMenuButton;
use self::components::table::Table;
use self::components::table_cell::TableCell;
use self::components::table_header::TableHeader;
use self::components::target_menu_button::TargetMenuButton;
use self::state::LogsState;
use crate::string::Ellipse;
use crate::time::DateTimeFormatExt;
use crate::tracing::collector::EventCollector;
use crate::tracing::CollectedEvent;

pub struct Logs {
    collector: EventCollector,
    state: RefCell<LogsState>,
}

impl Logs {
    #[must_use]
    pub fn new(collector: EventCollector) -> Self {
        Self { collector, state: Default::default() }
    }

    fn refresh_globset(&self) {
        let globs = self.state.borrow().target_filter.targets.clone();
        self.state.borrow_mut().target_filter.glob.replace({
            let mut glob = GlobSetBuilder::new();
            for target in globs {
                glob.add(target);
            }
            glob.build().unwrap()
        });
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Response {
        if self.state.borrow().target_filter.glob.is_none() {
            self.refresh_globset();
        }

        let row_height = constants::SEPARATOR_SPACING
            + ui.style().text_styles.get(&TextStyle::Small).unwrap().size;

        Table::default()
            .on_clear(|| {
                self.collector.clear();
            })
            .header(|ui| {
                TableHeader::default()
                    .common_props(CommonProps::new().min_width(100.0))
                    .children(|ui| {
                        ui.label("Time");
                    })
                    .show(ui);
                TableHeader::default()
                    .common_props(CommonProps::new().min_width(80.0))
                    .children(|ui| {
                        LevelMenuButton::default()
                            .state(&mut self.state.borrow_mut().level_filter)
                            .show(ui);
                    })
                    .show(ui);
                TableHeader::default()
                    .common_props(CommonProps::new().min_width(120.0))
                    .children(|ui| {
                        let changed = TargetMenuButton::default()
                            .state(&mut self.state.borrow_mut().target_filter)
                            .show(ui);

                        if changed {
                            self.refresh_globset();
                        }
                    })
                    .show(ui);
                TableHeader::default()
                    .common_props(CommonProps::new().min_width(120.0))
                    .children(|ui| {
                        ui.label("Message");
                    })
                    .show(ui);
            })
            .row_height(row_height)
            .row(|ui, event: &CollectedEvent| {
                TableCell::default()
                    .common_props(CommonProps::new().min_width(100.0))
                    .children(|ui| {
                        ui.label(event.time.format_short())
                            .on_hover_text(event.time.format_detailed());
                    })
                    .show(ui);
                TableCell::default()
                    .common_props(CommonProps::new().min_width(80.0))
                    .children(|ui| {
                        ui.colored_label(event.level.to_color32(), event.level.as_str());
                    })
                    .show(ui);
                TableCell::default()
                    .common_props(CommonProps::new().min_width(120.0))
                    .children(|ui| {
                        ui.label(event.target.truncate_graphemes(18))
                            .on_hover_text(&event.target);
                    })
                    .show(ui);
                TableCell::default()
                    .common_props(CommonProps::new().min_width(120.0))
                    .children(|ui| {
                        let mut short_message = String::new();
                        let mut complete_message = String::new();
                        let mut log_message = String::new();

                        if let Some(msg) = event.fields.get("message") {
                            let msg = msg.trim();
                            short_message.push_str(msg);
                            complete_message.push_str(msg);
                        }

                        for (key, value) in &event.fields {
                            if *key == "message" {
                                continue;
                            }
                            if key.starts_with("log.") {
                                log_message.push_str("\n ");
                                log_message.push_str(key);
                                log_message.push_str(": ");
                                log_message.push_str(value);
                            } else {
                                short_message.push_str(", ");
                                short_message.push_str(key);
                                short_message.push_str(": ");
                                short_message.push_str(value);
                                complete_message.push_str("\n ");
                                complete_message.push_str(key);
                                complete_message.push_str(": ");
                                complete_message.push_str(value);
                            }
                        }

                        complete_message.push_str("\n\n");
                        complete_message.push_str(&log_message);

                        ui.add(Label::new(short_message).wrap_mode(TextWrapMode::Extend))
                            .on_hover_text(complete_message);
                    })
                    .show(ui);
            })
            .show(ui, self.collector.events()
                  .iter()
                  .filter(|event| self.state.borrow().level_filter.get(event.level)
                          && !self.state.borrow().target_filter.glob.as_ref().is_some_and(|g| g.is_match(&event.target)))
                  .collect::<Vec<_>>()
                  .into_iter())
    }
}
