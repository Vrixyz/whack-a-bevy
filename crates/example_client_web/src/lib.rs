mod cheatbook;

use bevy::prelude::*;

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Deserializer;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

use example_shared::{ClientMessage, ServerMessage};

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
pub fn start_websocket() -> Result<(), JsValue> {
    // Connect to an echo server
    let ws = WebSocket::new("ws://127.0.0.1:8083")?;
    // For small binary messages, like CBOR, Arraybuffer is more efficient than Blob handling
    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
    // create callback
    let cloned_ws = ws.clone();
    let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
        // Handle difference Text/Binary,...
        if let Ok(abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
            console_log!("message event, received arraybuffer: {:?}", abuf);
            let array = js_sys::Uint8Array::new(&abuf);
            let len = array.byte_length() as usize;

            if let Ok(de) = serde_json::from_slice::<ServerMessage>(&array.to_vec()) {
                console_log!("Message received {:?}", de);
            }

            let mut buf = Vec::new();
            if ClientMessage::Position(example_shared::Vec2 {
                x: 0.5f32,
                y: 0.5f32,
            })
            .serialize(&mut serde_json::Serializer::new(&mut buf))
            .is_ok()
            {
                match cloned_ws.send_with_u8_array(&buf) {
                    Ok(_) => console_log!("binary message successfully sent"),
                    Err(err) => console_log!("error sending message: {:?}", err),
                }
            }
        } else {
            console_log!("message event, received Unknown: {:?}", e.data());
        }
    }) as Box<dyn FnMut(MessageEvent)>);
    // set message event handler on WebSocket
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    // forget the callback to keep it alive
    onmessage_callback.forget();

    let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
        console_log!("error event: {:?}", e);
    }) as Box<dyn FnMut(ErrorEvent)>);
    ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();

    let cloned_ws = ws.clone();
    let onopen_callback = Closure::wrap(Box::new(move |_| {
        // send off binary message
        let mut buf = Vec::new();
        if ClientMessage::Position(example_shared::Vec2 { x: 1f32, y: 1f32 })
            .serialize(&mut serde_json::Serializer::new(&mut buf))
            .is_ok()
        {
            match cloned_ws.send_with_u8_array(&buf) {
                Ok(_) => console_log!("binary message successfully sent"),
                Err(err) => console_log!("error sending message: {:?}", err),
            }
        }
    }) as Box<dyn FnMut(JsValue)>);
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();

    Ok(())
}
