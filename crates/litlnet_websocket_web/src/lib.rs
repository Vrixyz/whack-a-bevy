use std::borrow::BorrowMut;
use std::sync::Mutex;
use std::sync::Once;

use litlnet_trait::Communication;
use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};
// https://www.sitepoint.com/rust-global-variables/

static mut WEBSOCKET: Option<Mutex<Option<WebSocket>>> = None;
static WEBSOCKET_INIT: Once = Once::new();

fn global_websocket<'a>() -> &'a Mutex<Option<WebSocket>> {
    WEBSOCKET_INIT.call_once(|| unsafe {
        *WEBSOCKET.borrow_mut() = Some(Mutex::new(None));
    });
    unsafe { WEBSOCKET.as_ref().unwrap() }
}

static mut RECV_PACKETS: Option<Mutex<Vec<Vec<u8>>>> = None;
static RECV_PACKETS_INIT: Once = Once::new();
fn global_recv_packets<'a>() -> &'a Mutex<Vec<Vec<u8>>> {
    RECV_PACKETS_INIT.call_once(|| unsafe {
        *RECV_PACKETS.borrow_mut() = Some(Mutex::new(vec![]));
    });
    unsafe { RECV_PACKETS.as_ref().unwrap() }
}

pub struct WebsocketClient {}

impl WebsocketClient {
    pub fn connect(remote_addr: &str) -> Result<Self, std::io::Error> {
        if let Ok(websocket) = start_websocket(remote_addr) {
            if let Ok(mut ws) = global_websocket().lock() {
                if ws.is_none() {
                    *ws = Some(websocket);
                    return Ok(Self {});
                }
            }
        }
        todo!("connect failure to {}", remote_addr)
    }
}

impl Communication for WebsocketClient {
    fn receive<T: DeserializeOwned>(&mut self) -> Result<Option<Vec<T>>, std::io::Error> {
        match global_recv_packets().lock() {
            Ok(mut recv) => {
                if recv.is_empty() {
                    return Ok(None);
                }
                let mut res = vec![];
                for m in recv.iter() {
                    if let Ok(de) = serde_json::from_slice::<T>(m) {
                        res.push(de);
                    }
                }
                recv.clear();
                return Ok(Some(res));
            }
            Err(e) => todo!("receive failure{}", e),
        }
    }

    fn send<T: Serialize>(&mut self, message: &T) -> std::io::Result<()> {
        match global_websocket().lock() {
            Ok(ws) => match *ws {
                Some(ref ws) => {
                    let mut buf = Vec::new();
                    if message
                        .serialize(&mut serde_json::Serializer::new(&mut buf))
                        .is_ok()
                    {
                        match ws.send_with_u8_array(&buf) {
                            Ok(_) => {
                                dbg!("binary message successfully sent");
                            }
                            Err(err) => {
                                dbg!("error sending message: {:?}", err);
                            }
                        }
                    }
                }
                None => {
                    todo!("no socket at this point ?")
                }
            },
            Err(_) => {
                todo!("cannot lock websocket global")
            }
        }
        Ok(())
    }
}

fn start_websocket(remote_addr: &str) -> Result<WebSocket, JsValue> {
    // Connect to an echo server
    let ws = WebSocket::new(remote_addr)?;
    // For small binary messages, like CBOR, Arraybuffer is more efficient than Blob handling
    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
    // create callback
    let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
        // Handle difference Text/Binary,...
        if let Ok(abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
            dbg!("message event, received arraybuffer: {:?}", &abuf);
            let array = js_sys::Uint8Array::new(&abuf);
            match global_recv_packets().lock() {
                Ok(mut recv) => recv.push(array.to_vec()),
                Err(e) => todo!("cannot lock recv global ?"),
            }
        } else {
            dbg!("message event, received Unknown: {:?}", e.data());
        }
    }) as Box<dyn FnMut(MessageEvent)>);
    // set message event handler on WebSocket
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    // forget the callback to keep it alive
    onmessage_callback.forget();

    let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
        dbg!("error event: {:?}", e);
    }) as Box<dyn FnMut(ErrorEvent)>);
    ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();

    let cloned_ws = ws.clone();
    let onopen_callback = Closure::wrap(Box::new(move |_| {
        // ?
    }) as Box<dyn FnMut(JsValue)>);
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();

    Ok(ws)
}
