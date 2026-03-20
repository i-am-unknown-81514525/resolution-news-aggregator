use egui::Color32;
use common::unify::UnifyOutput;
use egui::containers::Frame;
use epaint::{Stroke, CornerRadius};
pub struct UnifyOutputDisplay(UnifyOutput);

impl egui::Widget for UnifyOutputDisplay {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.add(
            Frame {
                inner_margin: Default::default(),
                fill: Color32::from_rgb(16, 16, 16),
                stroke: Stroke::new(2.0f32, Color32::from_rgb(32, 32, 32)),
                corner_radius: CornerRadius::same(6),
                outer_margin: Default::default(),
                shadow: Default::default(),
            }.show(ui, |ui| {
                ui.hyperlink_to(self.0.title, self.0.link);
                if let Some(v) = (self.0.description) {
                    ui.label(&v);
                }
            })
        );
    }
}