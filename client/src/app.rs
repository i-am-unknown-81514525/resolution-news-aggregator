use std::fmt::Display;
use egui::{Color32, RichText};
use epaint::CornerRadius;
use uuid::Uuid;

#[cfg(target_arch = "wasm32")]
use web_sys::WebSocket;
use common::unify::{SourceKind, UnifyOutput};
use crate::schema::WindowConfig;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.

struct Internal {
    #[cfg(target_arch = "wasm32")]
    pub websocket: Option<WebSocket>,
}

impl Internal {
    fn new() -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            websocket: None,
        }
    }
}


#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    pub src: String,

    pub windows: Vec<WindowConfig>,

    pub history: Vec<UnifyOutput>,

    #[serde(skip)] 
    pub internal: Internal
}

impl Default for App {
    fn default() -> Self {
        Self {
            src: "".to_string(),
            windows: vec![WindowConfig::default()],
            history: Vec::new(),
            internal: Internal::new(),
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        let result;
        if let Some(storage) = cc.storage {
            result = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        } else {
            result = Default::default();
        }
        result
    }
}

impl eframe::App for App {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        for window in self.windows.clone() {
            egui::Window::new("News Panel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for item in self.history.iter().clone() {
                        egui::containers::Frame::new()
                            .corner_radius(CornerRadius::same(6))
                            .show(ui, |ui| {
                            ui.hyperlink_to(item.title, item.link.clone());
                            if item.description.len() > 0 {
                                ui.label(&item.description);
                            };
                            let mut tiny_text = String::new();
                            tiny_text.push_str(&item.organisation);
                            if let SourceKind::Source(x) = item.source.clone() {
                                tiny_text.push_str(&" - ");
                                tiny_text.push_str(&x);
                            }
                            ui.label(RichText::new(tiny_text).color(Color32::from_rgb(128, 128, 128)).size(3.0f32));
                        }).inner
                    }
                }).inner
            });
        }
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui


        // egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        //     // The top panel is often a good place for a menu bar:
        // 
        //     egui::MenuBar::new().ui(ui, |ui| {
        //         // NOTE: no File->Quit on web pages!
        //         let is_web = cfg!(target_arch = "wasm32");
        //         if !is_web {
        //             ui.menu_button("File", |ui| {
        //                 if ui.button("Quit").clicked() {
        //                     ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        //                 }
        //             });
        //             ui.add_space(16.0);
        //         }
        // 
        //         egui::widgets::global_theme_preference_buttons(ui);
        //     });
        // });

        // egui::CentralPanel::default().show(ctx, |ui| {
        //     // The central panel the region left after adding TopPanel's and SidePanel's
        //     ui.heading("eframe template");
        // 
        //     ui.horizontal(|ui| {
        //         ui.label("Write something: ");
        //         ui.text_edit_singleline(&mut self.label);
        //     });
        // 
        //     ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
        //     if ui.button("Increment").clicked() {
        //         self.value += 1.0;
        //     }
        // 
        //     ui.separator();
        // 
        //     ui.add(egui::github_link_file!(
        //         "https://github.com/emilk/eframe_template/blob/main/",
        //         "Source code."
        //     ));
        // 
        //     ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        //         powered_by_egui_and_eframe(ui);
        //         egui::warn_if_debug_build(ui);
        //     });
        // });

    }
}

// fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
//     ui.horizontal(|ui| {
//         ui.spacing_mut().item_spacing.x = 0.0;
//         ui.label("Powered by ");
//         ui.hyperlink_to("egui", "https://github.com/emilk/egui");
//         ui.label(" and ");
//         ui.hyperlink_to(
//             "eframe",
//             "https://github.com/emilk/egui/tree/master/crates/eframe",
//         );
//         ui.label(".");
//     });
// }