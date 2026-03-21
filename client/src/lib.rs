#![warn(clippy::all, rust_2018_idioms)]

mod app;

#[cfg(target_arch = "wasm32")]
mod wasm_websocket;
mod dt;
// mod comp;

pub use app::App;