use std::collections::VecDeque;
use egui::{Context, Id, InnerResponse, Ui};
use egui::scroll_area::ScrollBarVisibility;
use serde::{Deserialize, Serialize};
use common::unify::UnifyOutput;
use crate::comp::news_frame::NewsFrame;
use crate::comp::{CtxObj, UiObj};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FilterOption {
    NotVisible,
    Visible(Option<String>)
}

impl Default for FilterOption {
    fn default() -> Self {
        FilterOption::Visible(None)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Windows {
    pub id: u32,
    pub name: String,
    pub filters: FilterOption,
    pub can_close: bool,
    pub is_open: bool,
    #[serde(skip)]
    pub matched: Option<VecDeque<UnifyOutput>>
}

impl CtxObj for Windows {
    fn show(&mut self, ctx: &mut Context) -> () {
        let now = chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
        let mut base = egui::Window::new(self.name.clone())
            .id(Id::new(self.id))
            .scroll([false, true])
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden);
        if self.can_close {
            base = base.open(&mut self.is_open);
        }
        base.show(ctx, |ui| {
                ui.vertical(|ui| {
                    if let Some(v) = self.matched.clone() {
                        for item in v
                        {
                            if (now < item.time) {
                                continue;
                            }
                            NewsFrame(item.clone()).show(ui);
                        }
                    }
                }).inner
            });
    }
}