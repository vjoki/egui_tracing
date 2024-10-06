use egui::{Label, TextWrapMode, Ui};
use globset::Glob;

use crate::ui::state::TargetFilter;

#[derive(Default)]
pub struct TargetMenuButton<'a> {
    state: Option<&'a mut TargetFilter>,
}

impl<'a> TargetMenuButton<'a> {
    pub fn state(mut self, v: &'a mut TargetFilter) -> Self {
        self.state = Some(v);
        self
    }

    pub fn show(self, ui: &mut Ui) -> bool {
        let state = self.state.unwrap();
        let mut changed = false;
        ui.menu_button("Target", |ui| {
            ui.label("Target Filter");

            let (input, add_button) = ui
                .horizontal(|ui| {
                    let input = ui
                        .text_edit_singleline(&mut state.input)
                        .on_hover_text("example: eframe::*");
                    let button = ui.button("Add");
                    (input, button)
                })
                .inner;

            if add_button.clicked()
                || (input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
            {
                state.targets.push(Glob::new(&state.input).unwrap());
                state.input.clear();
                changed = true;
            }

            state.targets.retain(|target| {
                ui.separator();
                let resp = ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    let res = if ui.button("Delete").clicked() {
                        changed = true;
                        false
                    } else { true };
                    ui.add(Label::new(target.glob()).wrap_mode(TextWrapMode::Truncate));
                    res
                });
                resp.inner
            });
        });
        changed
    }
}
