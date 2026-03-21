use std::fmt::Display;
use egui::{Align, Color32, RichText};
use epaint::{CornerRadius, FontFamily, FontId};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use cfg_if::cfg_if;
use eframe::CreationContext;
use egui::scroll_area::ScrollBarVisibility;
use epaint::text::{LayoutJob, TextFormat, TextWrapping};
use indexmap::IndexMap;

#[cfg(target_arch = "wasm32")]
use crate::wasm_websocket::WasmWebsocket;


use common::unify::{SourceKind, UnifyOutput};
use crate::dt::format_fuzzy_dist;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.

struct Internal {
    #[cfg(target_arch = "wasm32")]
    pub ws: Option<WasmWebsocket>,
    pub sender: Sender<UnifyOutput>,
    pub receiver: Arc<Mutex<Receiver<UnifyOutput>>>,
    pub initial: bool
}

impl Internal {
    fn new() -> Self {
        let (sender, receiver) = channel();
        Self {
            #[cfg(target_arch = "wasm32")]
            ws: None,
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
            initial: true
        }
    }
}


#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    pub src: String,

    pub history: IndexMap<String, UnifyOutput>,

    #[serde(skip)] 
    pub internal: Internal
}

impl Default for App {
    fn default() -> Self {
        Self {
            src: "".to_string(),
            history: IndexMap::new(),
            internal: Internal::new(),
        }
    }
}
impl App {
    /// Called once before the first frame.
    pub fn new(cc: &CreationContext) -> Self {
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
        let url = "/NotoSerifCJK-VF.otf.ttc";

        let ctx = cc.egui_ctx.clone();

        ehttp::fetch(ehttp::Request::get(url), move |result| {
            if let Ok(response) = result {
                let font_bytes = response.bytes;
                let mut fonts = egui::FontDefinitions::default();

                // Insert the CJK font data
                fonts.font_data.insert(
                    "NotoSansCJKjp".to_owned(),
                    Arc::from(egui::FontData::from_owned(font_bytes)),
                );

                // Add it as the primary font for Proportional text
                fonts.families
                    .get_mut(&egui::FontFamily::Proportional)
                    .unwrap()
                    .push("NotoSansCJKjp".to_owned());

                ctx.set_fonts(fonts);
                ctx.request_repaint();
            }
        });
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
        let mut update: Vec<UnifyOutput> = Vec::new();
        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                if self.internal.ws.is_none() {
                    let mut path = String::from(&self.src);
                    path.push_str("/ws");
                    let ws = WasmWebsocket::new(&path, self.internal.sender.clone());
                    self.internal.ws = Some(ws);
                }
            }
        }
        if self.internal.initial {
            self.internal.initial = false;
            ctx.request_repaint();
            ctx.request_repaint_after(Duration::new(0, 200_000_000));
        }
        while let Ok(v) = self.internal.receiver.lock().unwrap().try_recv() {
            update.push(v);
        }
        let has_update = update.len() > 0;
        for item in update {
            self.history.entry(item.id.clone()).or_insert(item);
        }

        self.history.sort_by(|k1, v1, k2, v2| v1.time.timestamp_micros().cmp(&v2.time.timestamp_micros()));

        egui::Window::new("News Panel")
            .scroll([false, true])
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
            .show(ctx, |ui| {
            ui.vertical(|ui| {
                for item in self.history.iter().map(|x| x.1).rev()
                {
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
                                layout.append(&item.title, 0.0, TextFormat {
                                    font_id: FontId::new(14.0, FontFamily::Proportional),
                                    color: ui.visuals().hyperlink_color,
                                    ..Default::default()
                                },);

                                ui.add(
                                    egui::Hyperlink::from_label_and_url(
                                        layout,
                                        item.link.clone()
                                    ).open_in_new_tab(true)
                                );
                                if item.description.len() > 0 {
                                    ui.label(egui::RichText::new(item.description.clone()).size(11.0f32));
                                };
                                let mut tiny_text = String::new();
                                tiny_text.push_str(&item.organisation);
                                if let SourceKind::Source(x) = item.source.clone() {
                                    tiny_text.push_str(&" via ");
                                    tiny_text.push_str(&x);
                                };
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(tiny_text).color(Color32::from_rgb(128, 128, 128)).size(9.0f32));
                                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                                        let time = ui.label(format_fuzzy_dist(item.time));
                                        if time.clicked() {
                                            ctx.copy_text(item.time.to_rfc3339());
                                        }
                                    });
                                });
                            }).inner
                        }).inner
                }
            }).inner
        });

        if has_update {
            ctx.request_repaint();
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