mod cheatbook;

use std::borrow::BorrowMut;
use std::collections::VecDeque;
use std::sync::Mutex;
use std::sync::Once;

use bevy::prelude::*;

use litlnet_trait::Communication;
use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

use example_shared::{ClientMessage, ServerMessage};

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

pub struct MessagesToSend<S: Serialize> {
    messages: VecDeque<S>,
}

impl<S: Serialize> Default for MessagesToSend<S> {
    fn default() -> Self {
        Self {
            messages: VecDeque::new(),
        }
    }
}
impl<S: Serialize> MessagesToSend<S> {
    pub fn push(&mut self, message: S) {
        self.messages.push_back(message);
    }
}
pub struct MessagesToRead<R: DeserializeOwned> {
    messages: VecDeque<R>,
}

impl<R: DeserializeOwned> Default for MessagesToRead<R> {
    fn default() -> Self {
        Self {
            messages: VecDeque::new(),
        }
    }
}
impl<R: DeserializeOwned> MessagesToRead<R> {
    pub fn pop(&mut self) -> Option<R> {
        self.messages.pop_front()
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

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
struct VisualPlayer {
    id: usize,
}

struct AssetsVisualPlayer {
    pub sprite_handles: Vec<Handle<Image>>,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins);
        /*app.add_plugin(ClientPlugin::<
            Client,
            ClientMessage,
            ServerMessage,
        >::default());*/
        app.insert_resource(MessagesToSend::<ClientMessage>::default());
        app.insert_resource(MessagesToRead::<ServerMessage>::default());
        let client: Option<WebsocketClient> = None;
        app.insert_resource(client);
        app.add_startup_system(setup);
        app.add_system(send_messages);
        app.add_system(_send_messages::<WebsocketClient, ClientMessage>);
        app.add_system(receive_messages);
        app.add_system(_receive_messages::<WebsocketClient, ServerMessage>);
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    App::new().add_plugin(GamePlugin).run();
}

pub fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut client: ResMut<Option<WebsocketClient>>,
) {
    if let Ok(ws) = WebsocketClient::connect("ws://127.0.0.1:8083") {
        *client = Some(ws);
    }
    commands
        .spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);
    let sprite_handle = assets.load("players/icon_bevy.png");
    commands.insert_resource(AssetsVisualPlayer {
        sprite_handles: vec![sprite_handle],
    });
}

fn send_messages(
    mut send: ResMut<MessagesToSend<ClientMessage>>,
    buttons: Res<Input<MouseButton>>,
    wnds: Res<Windows>,
    q_camera: Query<&Transform, With<MainCamera>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        if let Ok(world_position) = cheatbook::cursor_to_world(&wnds, q_camera.single()) {
            send.push(ClientMessage::Position(example_shared::Vec2 {
                x: world_position.x,
                y: world_position.y,
            }));
        }
    }
}
fn receive_messages(
    mut commands: Commands,
    sprites: Res<AssetsVisualPlayer>,
    mut recv: ResMut<MessagesToRead<ServerMessage>>,
    mut units: Query<(&mut Transform, &VisualPlayer)>,
) {
    while let Some(ServerMessage::Spawn(message)) = recv.pop() {
        let mut unit_found_and_moved = false;
        for (mut t, v) in units.iter_mut() {
            if v.id == message.client_id {
                t.translation = Vec3::new(message.position.x, message.position.y, 0f32);
                unit_found_and_moved = true;
                break;
            }
        }
        if !unit_found_and_moved {
            commands.spawn().insert_bundle(SpriteBundle {
                texture: sprites.sprite_handles[0].clone(),
                transform: Transform::from_translation(Vec3::new(
                    message.position.x,
                    message.position.y,
                    0f32,
                )),
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(64.0)),
                    ..Default::default()
                },
                ..Default::default()
            });
        }
    }
}

pub struct WebsocketClient {}

impl WebsocketClient {
    pub fn connect(remote_addr: &str) -> Result<Self, std::io::Error> {
        if let Ok(websocket) = start_websocket() {
            if let Ok(mut ws) = global_websocket().lock() {
                if ws.is_none() {
                    *ws = Some(websocket);
                    return Ok(Self {});
                }
            }
        }
        todo!()
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
            Err(_) => todo!(),
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
                            Ok(_) => console_log!("binary message successfully sent"),
                            Err(err) => console_log!("error sending message: {:?}", err),
                        }
                    }
                }
                None => {
                    todo!()
                }
            },
            Err(_) => {
                todo!()
            }
        }
        Ok(())
    }
}
fn _receive_messages<
    C: Communication + Send + Sync + 'static,
    R: DeserializeOwned + Send + Sync + 'static,
>(
    mut com: ResMut<Option<C>>,
    mut messages_to_read: ResMut<MessagesToRead<R>>,
) {
    if let Some(com) = com.as_mut() {
        match com.receive() {
            Ok(Some(messages)) => {
                for message in messages {
                    messages_to_read.messages.push_back(message);
                }
            }
            Ok(None) => {}
            Err(_e) => {
                //dbg!(e);
            }
        }
    }
}
fn _send_messages<
    C: Communication + Send + Sync + 'static,
    S: Serialize + Send + Sync + 'static,
>(
    mut com: ResMut<Option<C>>,
    mut messages_to_send: ResMut<MessagesToSend<S>>,
) {
    let mut is_fail = false;
    if let Some(com) = com.as_mut() {
        for msg in messages_to_send.messages.iter() {
            if com.send(&msg).is_err() {
                is_fail = true;
            }
        }
        messages_to_send.messages.clear();
    }
    if is_fail {
        *com = None;
    }
}

pub fn start_websocket() -> Result<WebSocket, JsValue> {
    // Connect to an echo server
    let ws = WebSocket::new("ws://127.0.0.1:8083")?;
    // For small binary messages, like CBOR, Arraybuffer is more efficient than Blob handling
    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
    // create callback
    let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
        // Handle difference Text/Binary,...
        if let Ok(abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
            console_log!("message event, received arraybuffer: {:?}", abuf);
            let array = js_sys::Uint8Array::new(&abuf);
            match global_recv_packets().lock() {
                Ok(mut recv) => recv.push(array.to_vec()),
                Err(_) => todo!(),
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

    Ok(ws)
}
