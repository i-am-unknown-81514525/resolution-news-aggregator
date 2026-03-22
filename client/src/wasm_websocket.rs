use std::sync::{Arc, Mutex, RwLock};
use wasm_bindgen::prelude::*;
use web_sys::{CloseEvent, ErrorEvent, MessageEvent, WebSocket};
use std::sync::mpsc::{Sender};
use js_sys::JsString;
use js_sys::wasm_bindgen::closure::Closure;
use web_sys::wasm_bindgen::JsCast;
use common::unify::UnifyOutput;
use serde_json::{from_str};
use crate::app::Internal;

pub struct WasmWebsocket(pub WebSocket, pub Sender<UnifyOutput>);


macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

impl WasmWebsocket {
    pub(crate) fn new(path: &str, sender: Sender<UnifyOutput>, internal: Arc<RwLock<Internal>>) -> WasmWebsocket {
        let ws = WebSocket::new(path).unwrap();
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
        // create callback
        let cloned_ws = ws.clone();
        let inner = Arc::new(Mutex::new(sender.clone()));
        let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
            // Handle difference Text/Binary,...
            let string: String;
            if let Ok(abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
                console_log!("message event, received arraybuffer: {:?}", abuf);
                let array = js_sys::Uint8Array::new(&abuf);
                string = match String::from_utf8(array.to_vec()) {
                    Ok(s) => s,
                    Err(e) => {
                        console_log!("Fail to convert to utf-8");
                        return;
                    }
                };

            }
            else if let Ok(s) = e.data().dyn_into::<js_sys::JsString>() {
                string = s.into();
            } else {
                return;
            }
            let content: UnifyOutput = match serde_json::from_str(&string) {
                Ok(s) => s,
                Err(e) => {
                    console_log!("Fail to deserialize json");
                    return;
                }
            };
            match inner.lock().unwrap().send(content) {
                Ok(_) => {},
                Err(e) => {
                    console_log!("Fail to contact frontend")
                }
            }
        });
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();
        let cloned_internal = internal.clone();
        let on_err_callback = Closure::<dyn FnMut(_)>::new(move |_e: ErrorEvent| {
            cloned_internal.write().unwrap().ws = None;
            console_log!("websocket disconnection by error: {:?}", _e);
        });
        ws.set_onerror(Some(on_err_callback.as_ref().unchecked_ref()));
        on_err_callback.forget();
        let on_close_callback = Closure::<dyn FnMut(_)>::new(move |_e: CloseEvent| {
            internal.write().unwrap().ws = None;
            console_log!("websocket disconnection: {:?}", _e);
        });
        ws.set_onclose(Some(on_close_callback.as_ref().unchecked_ref()));
        on_close_callback.forget();
        WasmWebsocket(ws, sender)
    }
}

