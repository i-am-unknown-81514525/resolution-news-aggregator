use std::fmt::Display;
use egui::{Align, Color32, RichText};
use epaint::{CornerRadius, FontFamily, FontId};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use cfg_if::cfg_if;
use eframe::CreationContext;
use egui::scroll_area::ScrollBarVisibility;
use epaint::text::{LayoutJob, TextFormat, TextWrapping};
use indexmap::IndexMap;
use crate::utils::truncate_text;

#[cfg(target_arch = "wasm32")]
use crate::wasm_websocket::WasmWebsocket;


use common::unify::{SourceKind, UnifyOutput};
use crate::dt::format_fuzzy_dist;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.

struct Count(Option<u32>);

pub struct Internal {
    #[cfg(target_arch = "wasm32")]
    pub ws: Option<WasmWebsocket>,
    pub sender: Sender<UnifyOutput>,
    pub receiver: Arc<Mutex<Receiver<UnifyOutput>>>,
    pub initial: bool,
    pub page: Arc<RwLock<Count>>,
    // pub last_update: chrono::DateTime<chrono::FixedOffset>
}

impl Internal {
    fn new() -> Self {
        let (sender, receiver) = channel();
        Self {
            #[cfg(target_arch = "wasm32")]
            ws: None,
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
            initial: true,
            page: Arc::new(RwLock::new(Count(Some(0)))),
            // last_update: chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap())
        }
    }
}


#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    pub src: String,

    pub history: Arc<RwLock<IndexMap<String, UnifyOutput>>>,

    #[serde(skip)] 
    pub internal: Arc<RwLock<Internal>>
}

impl Default for App {
    fn default() -> Self {
        Self {
            src: "".to_string(),
            history: Arc::new(RwLock::new(IndexMap::new())),
            internal: Arc::new(RwLock::new(Internal::new())),
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
        let result: App;
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

        // let mut history: String = result.src.clone();
        // history.push_str(&"/api/history");
        // let history_rw = result.history.clone();
        // ehttp::fetch(ehttp::Request::get(history), move |result| {
        //     if let Ok(response) = result {
        //         if let Some(data) = response.text() {
        //             let out: Vec<UnifyOutput> = serde_json::from_str(data).unwrap();
        //             for item in out {
        //                 history_rw.write().unwrap().entry(item.id.clone()).or_insert(item);
        //             }
        //         }
        //     }
        // });
        update_feed(cc.egui_ctx.clone(), result.history.clone(), result.internal.read().unwrap().page.clone(), &result.src);
        result
    }
}

fn update_feed(ctx: egui::Context, rw: Arc<RwLock<IndexMap<String, UnifyOutput>>>, counter: Arc<RwLock<Count>>, path: &str) {
    let page_num: u32 = {
        let mut lock = counter.write().unwrap();
        let v = match lock.0 {
            None => return (),
            Some(v) => v,
        };
        lock.0 = None;
        v
    };
    let mut history: String = path.to_string();
    history.push_str(&"/api/history");
    history.push_str(&format!("?page={}", page_num));
    let counter_clone = counter.clone();
    ehttp::fetch(ehttp::Request::get(history), move |result| {
        if let Ok(response) = result {
            if let Some(data) = response.text() {
                let out: Vec<UnifyOutput> = serde_json::from_str(data).unwrap();
                if out.len() > 0 {
                    let mut lock = counter_clone.write().unwrap();
                    lock.0 = Some(page_num + 1);
                } else {
                    let mut lock = counter_clone.write().unwrap();
                    lock.0 = None;
                }
                for item in out {
                    rw.write().unwrap().entry(item.id.clone()).or_insert(item);
                }
                ctx.request_repaint();
            }
        }
    });
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
                if self.internal.read().unwrap().ws.is_none() {
                    let mut path = String::from(&self.src);
                    path.push_str("/ws");
                    let ws = WasmWebsocket::new(&path, self.internal.read().unwrap().sender.clone(), self.internal.clone());
                    self.internal.write().unwrap().ws = Some(ws);
                    self.internal.write().unwrap().page.clone().write().unwrap().0 = Some(0);
                }
            }
        }
        update_feed(ctx.clone(), self.history.clone(), self.internal.read().unwrap().page.clone(), &self.src);
        if self.internal.read().unwrap().initial {
            self.internal.write().unwrap().initial = false;
            ctx.request_repaint();
            ctx.request_repaint_after(Duration::new(0, 200_000_000));
        }
        while let Ok(v) = self.internal.read().unwrap().receiver.lock().unwrap().try_recv() {
            update.push(v);
        }
        let has_update = update.len() > 0;
        for item in update {
            self.history.write().unwrap().entry(item.id.clone()).or_insert(item);
        }
        // if has_update {
        //     self.internal.write().unwrap().last_update = chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
        // } else {
        //     let curr = chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
        //     if (curr - self.internal.read().unwrap().last_update).as_seconds_f32() > 600f32 {
        //         let mut l = self.internal.write().unwrap();
        //         l.ws = None; // Reconnect
        //         l.page.write().unwrap().0 = Some(0);
        //         l.last_update = chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
        //     }
        // }

        self.history.write().unwrap().sort_by(|k1, v1, k2, v2| v1.time.timestamp_micros().cmp(&v2.time.timestamp_micros()));


        egui::Window::new("News Panel")
            .scroll([false, true])
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
            .show(ctx, |ui| {
            ui.vertical(|ui| {
                for item in self.history.read().unwrap().iter().map(|x| x.1).rev().collect::<Vec<&UnifyOutput>>()
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
                                    ui.label(egui::RichText::new(truncate_text(&item.description, 600)).size(11.0f32));
                                };
                                let mut tiny_text = String::new();
                                tiny_text.push_str(&item.organisation);
                                if let SourceKind::Source(x) = item.source.clone() {
                                    tiny_text.push_str(&" via ");
                                    tiny_text.push_str(&x);
                                } else if let SourceKind::LinkedSource(x, _) = item.source.clone() {
                                    tiny_text.push_str(&" via ");
                                    tiny_text.push_str(&x);
                                }
                                ui.horizontal(|ui| {
                                    if let SourceKind::LinkedSource(_, l) = item.source.clone() {
                                        ui.add(
                                            egui::Hyperlink::from_label_and_url(
                                                RichText::new(tiny_text)
                                                    .color(Color32::from_rgb(128, 128, 128))
                                                    .size(9.0f32),
                                                l
                                            ).open_in_new_tab(true)
                                        );
                                    } else {
                                        ui.label(RichText::new(tiny_text).color(Color32::from_rgb(128, 128, 128)).size(9.0f32));
                                    }
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
        ctx.request_repaint_after(Duration::new(15, 0)); // Repaint every 15s to update time

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