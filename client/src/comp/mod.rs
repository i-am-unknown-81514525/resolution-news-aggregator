use egui::{Context, InnerResponse, Ui};

pub mod windows;
pub mod news_frame;

pub trait UiObj {
    fn show(&mut self, ui: &mut Ui) -> ();
}

pub trait CtxObj {
    fn show(&mut self, ui: &mut Context) -> ();
}