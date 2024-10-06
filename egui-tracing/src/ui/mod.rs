mod color;
mod components;
mod state;

use std::cell::RefCell;

use egui::text::{LayoutJob, LayoutSection, TextWrapping};
use egui::{Label, Response, TextFormat, TextStyle, TextWrapMode};
use egui_extras::{Column, TableBuilder};
use globset::GlobSetBuilder;

use self::color::ToColor32;
use self::components::level_menu_button::LevelMenuButton;
use self::components::target_menu_button::TargetMenuButton;
use self::state::LogsState;
use crate::time::DateTimeFormatExt;
use crate::tracing::collector::EventCollector;

pub struct Logs {
    collector: EventCollector,
    state: RefCell<LogsState>,
    indices: Vec<usize>,
}

impl Logs {
    #[must_use]
    pub fn new(collector: EventCollector) -> Self {
        Self { state: Default::default(), indices: Vec::with_capacity(collector.max_events()), collector }
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

        let small_font_id = TextStyle::Body.resolve(ui.style());
        let row_height = small_font_id.size;

        ui.vertical(|ui| {
            let mut scroll_to_bottom = false;

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Clear").on_hover_text("Clear Events").clicked() {
                        self.collector.clear();
                    }

                    ui.separator();

                    if ui
                        .button("To Bottom")
                        .on_hover_text("Scroll to Bottom")
                        .clicked()
                    {
                        scroll_to_bottom = true;
                    }
                })
            });

            let mut table = TableBuilder::new(ui)
                .column(Column::auto().at_least(100.0).resizable(true))
                .column(Column::auto().at_least(80.0).resizable(true))
                .column(Column::initial(120.0).at_least(120.0).resizable(true))
                .column(Column::remainder().at_least(120.0).resizable(true))
                .vscroll(true)
                .auto_shrink([false, false])
                .stick_to_bottom(true);

            if scroll_to_bottom {
                table = table.scroll_to_row(self.collector.events().len(), Some(egui::Align::TOP));
            }

            table
                .header(row_height, |mut row| {
                    row.col(|ui| {
                        ui.label("Time");
                    });
                    row.col(|ui| {
                        let changed = LevelMenuButton::default()
                            .state(&mut self.state.borrow_mut().level_filter)
                            .show(ui);

                        if changed {
                            self.collector.set_max_filter_level(self.state.borrow().level_filter.max_level());
                        }
                    });
                    row.col(|ui| {
                        let changed = TargetMenuButton::default()
                            .state(&mut self.state.borrow_mut().target_filter)
                            .show(ui);

                        if changed {
                            self.refresh_globset();
                        }
                    });
                    row.col(|ui| {
                        ui.label("Message");
                    });
                })
                .body(|body| {
                    self.indices.clear();
                    let events = self.collector.events();
                    assert!(events.len() <= self.indices.capacity());
                    self.indices.extend(
                        events.iter()
                            .enumerate()
                            .filter_map(|(i, event)| {
                                let st = self.state.borrow();
                                if st.level_filter.get(event.level)
                                    && !st.target_filter.glob.as_ref().is_some_and(|g| g.is_match(&event.target)) {
                                        Some(i)
                                    } else { None }
                            })
                    );

                    let message_size = body.widths()[3];

                    body.rows(row_height, self.indices.len(), |mut row| {
                        assert!(row.index() < events.len());
                        if let Some(event) = events.get(self.indices.get(row.index()).copied().unwrap_or_default()) {
                            row.col(|ui| {
                                ui.label(event.time.format_short())
                                    .on_hover_text(event.time.format_detailed());
                            });

                            row.col(|ui| {
                                ui.colored_label(event.level.to_color32(), event.level.as_str());
                            });

                            row.col(|ui| {
                                ui.add(Label::new(&event.target).wrap_mode(TextWrapMode::Truncate));
                            });

                            row.col(|ui| {
                                let msg = event.fields.get("message").cloned().unwrap_or_default();
                                let mut job = LayoutJob {
                                    sections: vec![LayoutSection{
                                        leading_space: 0.0,
                                        byte_range: 0..msg.len(),
                                        format: TextFormat::simple(small_font_id.clone(), egui::Color32::WHITE),
                                    }],
                                    text: msg,
                                    wrap: TextWrapping::truncate_at_width(message_size),
                                    first_row_min_height: 0.0,
                                    break_on_newline: false,
                                    halign: egui::Align::LEFT,
                                    justify: true,
                                    round_output_size_to_nearest_ui_point: true,
                                };

                                for (key, value) in &event.fields {
                                    if *key == "message" {
                                        continue;
                                    }
                                    job.append(" ", 0.0, TextFormat::simple(small_font_id.clone(), egui::Color32::GRAY));
                                    job.append(key, 0.0, TextFormat::simple(small_font_id.clone(), egui::Color32::GRAY));
                                    job.append("=", 0.0, TextFormat::simple(small_font_id.clone(), egui::Color32::GRAY));
                                    job.append(value, 0.0, TextFormat::simple(small_font_id.clone(), egui::Color32::WHITE));
                                }

                                ui.add(Label::new(job).wrap_mode(TextWrapMode::Truncate));
                            });
                        }
                    })
            })
        }).response
    }
}
