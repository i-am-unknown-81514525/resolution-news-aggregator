use egui::Color32;
use common::unify::{SourceKind, UnifyOutput};
use egui::containers::Frame;
use egui::RichText;
use epaint::{Stroke, CornerRadius};

#[derive(Debug)]
pub struct UnifyOutputDisplay(pub UnifyOutput);

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
                if self.0.description.len() > 0 {
                    ui.label(&self.0.description);
                };
                let mut tiny_text = String::new();
                tiny_text.push_str(&self.0.organisation);
                if let SourceKind::Source(x) = self.0.source {
                    tiny_text.push_str(&" - ");
                        tiny_text.push_str(&x);
                }
                ui.label(RichText::new(tiny_text).color(Color32::from_rgb(128, 128, 128)).size(3.0f32));
            }).inner
        );
    }
}