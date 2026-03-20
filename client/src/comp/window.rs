use std::sync::{Arc, Mutex};
use crate::schema::WindowConfig;
use common::unify::UnifyOutput;
use crate::comp::news_frame::UnifyOutputDisplay;

#[derive(Debug)]
pub struct Window {
    config: Arc<Mutex<WindowConfig>>,
    render: Vec<UnifyOutputDisplay>
}

impl Window {
    pub fn new(config: Arc<Mutex<WindowConfig>>) -> Self {
        Self {
            config,
            render: Vec::new()
        }
    }

    pub fn push_one(&mut self, item: &UnifyOutput) {
        self.render.push(UnifyOutputDisplay(item.clone()));
    }

    pub fn push_multi(&mut self, items: &[UnifyOutput]) {
        for item in items {
            self.push_one(item);
        }
    }
}

impl egui::Widget for Window {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            for item in self.render.iter().clone() {
                ui.add(item);
            }
        }).inner
    }
}

