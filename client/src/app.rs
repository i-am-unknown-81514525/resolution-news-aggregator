use std::collections::VecDeque;
use std::fs::read;
use std::panic::resume_unwind;
use egui::{Align, Color32, PointerButton, RichText};
use epaint::{CornerRadius, FontFamily, FontId};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use cfg_if::cfg_if;
use eframe::CreationContext;
use egui::scroll_area::ScrollBarVisibility;
use epaint::text::{LayoutJob, TextFormat, TextWrapping};
use indexmap::IndexMap;
use wasm_bindgen::prelude::wasm_bindgen;
use crate::utils::truncate_text;

#[cfg(target_arch = "wasm32")]
use crate::wasm_websocket::WasmWebsocket;


use common::unify::{SourceKind, UnifyOutput};
use crate::comp::news_frame::NewsFrame;
use crate::comp;
use crate::dt::format_fuzzy_dist;

use dashmap::{DashMap, DashSet};
use crate::comp::windows::{FilterOption, Windows};
use crate::comp::CtxObj;

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

    pub windows: Arc<DashMap<u32, Arc<Mutex<Windows>>>>,

    #[serde(skip)] 
    pub internal: Arc<RwLock<Internal>>
}

impl Default for App {
    fn default() -> Self {
        let mut dashmap: DashMap<u32, Arc<Mutex<Windows>>> = DashMap::new();
        dashmap.entry(0).or_insert(Arc::new(Mutex::new(Windows {
            id: 0,
            name: "Unify".to_string(),
            filters: FilterOption::NotVisible,
            can_close: false,
            is_open: true,
            matched: None
        })));
        Self {
            src: "".to_string(),
            history: Arc::new(RwLock::new(IndexMap::new())),
            internal: Arc::new(RwLock::new(Internal::new())),
            windows: Arc::new(dashmap),
        }
    }
}
impl App {
    /// Called once before the first frame.
    pub fn new(cc: &CreationContext) -> Self {
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

                fonts.font_data.insert(
                    "NotoSansCJKjp".to_owned(),
                    Arc::from(egui::FontData::from_owned(font_bytes)),
                );

                fonts.families
                    .get_mut(&egui::FontFamily::Proportional)
                    .unwrap()
                    .push("NotoSansCJKjp".to_owned());

                ctx.set_fonts(fonts);
                ctx.request_repaint();
            }
        });
        update_feed(cc.egui_ctx.clone(), result.history.clone(), result.internal.read().unwrap().page.clone(), &result.src, result.windows.clone());
        result
    }
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}


fn update_feed(ctx: egui::Context, rw: Arc<RwLock<IndexMap<String, UnifyOutput>>>, counter: Arc<RwLock<Count>>, path: &str, windows: Arc<DashMap<u32, Arc<Mutex<Windows>>>>) {
    let page_num: u32 = {
        let mut lock = counter.write().unwrap();
        let v = match lock.0 {
            None => return ,
            Some(v) => v,
        };
        lock.0 = None;
        v
    };
    let mut history: String = path.to_string();
    history.push_str("/api/history");
    history.push_str(&format!("?page={}", page_num));
    let counter_clone = counter.clone();
    ehttp::fetch(ehttp::Request::get(history), move |result| {
        if let Ok(response) = result
            && response.status / 100 == 2
            && let Some(data) = response.text() {
                let out: Vec<UnifyOutput> = match serde_json::from_str(data) {
                    Ok(t)=>t,
                    Err(e) => {
                        console_log!("Failed to serialize, data: \'{}\', error: \'{}\'", data, e);
                        return;
                    }
                };
                if !out.is_empty() {
                    let mut lock = counter_clone.write().unwrap();
                    lock.0 = Some(page_num + 1);
                } else {
                    let mut lock = counter_clone.write().unwrap();
                    lock.0 = None;
                }
                for item in out {
                    let cl = windows.clone();
                    rw.write().unwrap().entry(item.id.clone()).or_insert_with(|| {
                        process(cl, item.clone());
                        item
                    });
                }
                ctx.request_repaint();
            }
    });
}

fn process(windows: Arc<DashMap<u32, Arc<Mutex<Windows>>>>, news: UnifyOutput) {
    for window in windows.iter() {
        let mut window = window.lock().unwrap();
        if let None = window.matched {
            window.matched = Some(VecDeque::new()); // temp
        }
        match window.filters.clone() {
            FilterOption::NotVisible | FilterOption::Visible(None) => {
                window.matched.as_mut().unwrap().push_front(news.clone());
            }
            FilterOption::Visible(Some(filter)) => {
                // TODO : filter logic
                window.matched.as_mut().unwrap().push_front(news.clone());
            }
        }
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
                if self.internal.read().unwrap().ws.is_none() {
                    let mut path = String::from(&self.src);
                    path.push_str("/ws");
                    let ws = WasmWebsocket::new(&path, self.internal.read().unwrap().sender.clone(), self.internal.clone());
                    self.internal.write().unwrap().ws = Some(ws);
                    self.internal.write().unwrap().page.clone().write().unwrap().0 = Some(0);
                }
            }
        }
        update_feed(ctx.clone(), self.history.clone(), self.internal.read().unwrap().page.clone(), &self.src, self.windows.clone());
        if self.internal.read().unwrap().initial {
            self.internal.write().unwrap().initial = false;
            ctx.request_repaint();
            ctx.request_repaint_after(Duration::new(0, 200_000_000));
        }
        while let Ok(v) = self.internal.read().unwrap().receiver.lock().unwrap().try_recv() {
            update.push(v);
        }
        let has_update = !update.is_empty();
        for item in update {
            let cl = self.windows.clone();
            self.history.write().unwrap().entry(item.id.clone()).or_insert_with(|| {
                process(cl, item.clone());
                item
            });
        }

        self.history.write().unwrap().sort_by(|_k1, v1, _k2, v2| v1.time.timestamp_micros().cmp(&v2.time.timestamp_micros()));

        for window in self.windows.iter() {
            window.lock().unwrap().show(&mut ctx.clone());
        }

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