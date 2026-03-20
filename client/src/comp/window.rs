use std::sync::{Arc, Mutex};
use crate::schema::WindowConfig;
use common::unify::UnifyOutput;

#[derive(Debug)]
struct Window {
    config: Arc<Mutex<WindowConfig>>,
    render: Vec<UnifyOutput>
}

impl egui::Widget for Window {
    fn ui(&mut self, ui: &mut egui::Ui) {
        // ui.add(egui::Window::new(
        //
        // ));
    }
}

