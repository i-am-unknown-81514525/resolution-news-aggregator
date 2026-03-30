use eframe::emath::Align;
use egui::{Color32, InnerResponse, RichText, Ui};
use epaint::{CornerRadius, FontFamily, FontId};
use epaint::text::{LayoutJob, TextFormat, TextWrapping};
use common::unify::{SourceKind, UnifyOutput};
use crate::comp::UiObj;
use crate::dt::format_fuzzy_dist;
use crate::utils::truncate_text;

pub struct NewsFrame(pub UnifyOutput);

impl UiObj for NewsFrame {
    fn show(&mut self, ui: &mut Ui) -> () {
        Some(
            egui::containers::Frame::new()
            .corner_radius(CornerRadius::same(6))
            .outer_margin(5.0)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    //ui.hyperlink_to(item.title.clone(), item.link.clone());
                    let mut layout = LayoutJob::default();
                    layout.wrap = TextWrapping::wrap_at_width(ui.available_width());
                    layout.round_output_to_gui = true;
                    layout.break_on_newline = true;
                    layout.halign = Align::LEFT;
                    layout.append(&self.0.title, 0.0, TextFormat {
                        font_id: FontId::new(14.0, FontFamily::Proportional),
                        color: ui.visuals().hyperlink_color,
                        ..Default::default()
                    },);

                    let link = ui.add(
                        egui::Hyperlink::from_label_and_url(
                            layout,
                            self.0.link.clone()
                        ).open_in_new_tab(true)
                    );
                    if link.secondary_clicked() {
                        ui.ctx().copy_text(self.0.link.clone());
                    }
                    if !self.0.description.is_empty() {
                        ui.label(egui::RichText::new(truncate_text(&self.0.description, 600)).size(11.0f32));
                    };
                    let mut tiny_text = String::new();
                    tiny_text.push_str(&self.0.organisation);
                    if let SourceKind::Source(x) = self.0.source.clone() {
                        tiny_text.push_str(" via ");
                        tiny_text.push_str(&x);
                    } else if let SourceKind::LinkedSource(x, _) = self.0.source.clone() {
                        tiny_text.push_str(" via ");
                        tiny_text.push_str(&x);
                    }
                    ui.horizontal(|ui| {
                        if let SourceKind::LinkedSource(_, l) = self.0.source.clone() {
                            let link = ui.add(
                                egui::Hyperlink::from_label_and_url(
                                    RichText::new(tiny_text)
                                        .color(Color32::from_rgb(128, 128, 128))
                                        .size(9.0f32),
                                    l.clone()
                                ).open_in_new_tab(true)
                            );
                            if link.secondary_clicked() {
                                ui.ctx().copy_text(l);
                            }
                        } else {
                            ui.label(RichText::new(tiny_text).color(Color32::from_rgb(128, 128, 128)).size(9.0f32));
                        }
                        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                            let time = ui.label(format_fuzzy_dist(self.0.time));
                            if time.clicked() {
                                ui.ctx().copy_text(self.0.time.to_rfc3339());
                            }
                        });
                    });
                }).inner
            }));
    }
}