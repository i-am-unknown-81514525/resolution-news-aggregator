#![warn(clippy::all, rust_2018_idioms)]

mod app;

#[cfg(target_arch = "wasm32")]
mod wasm_websocket;
mod dt;
mod utils;
mod comp;
mod local_unify;
// mod comp;
mod db;

pub use app::App;