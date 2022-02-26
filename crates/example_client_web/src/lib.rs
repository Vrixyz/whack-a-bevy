mod cheatbook;

use bevy::prelude::*;

use example_shared::{ClientMessage, ServerMessage};
use litlnet_client_bevy::{ClientPlugin, MessagesToRead, MessagesToSend};
use litlnet_websocket_web::WebsocketClient;
use wasm_bindgen::prelude::*;

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
struct VisualMole {
    id: usize,
}

struct AssetsVisualPlayer {
    pub sprite_handles: Vec<Handle<Image>>,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins);
        app.add_plugin(ClientPlugin::<WebsocketClient, ClientMessage, ServerMessage>::default());
        app.add_startup_system(setup);
        app.add_system(send_messages);
        app.add_system(receive_messages);
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
            send.push(ClientMessage::HitPosition(Vec2::new(
                world_position.x,
                world_position.y,
            )));
        }
    }
}
fn receive_messages(
    mut commands: Commands,
    sprites: Res<AssetsVisualPlayer>,
    mut recv: ResMut<MessagesToRead<ServerMessage>>,
    mut moles: Query<(Entity, &Transform, &VisualMole)>,
) {
    while let Some(message) = recv.pop() {
        match message {
            ServerMessage::Spawn(spawn) => {
                dbg!("new mole: {spawn}");
                commands.spawn().insert_bundle(SpriteBundle {
                    texture: sprites.sprite_handles[0].clone(),
                    transform: Transform::from_translation(spawn.def.position.extend(0f32)),
                    sprite: Sprite {
                        custom_size: Some(Vec2::splat(64.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                });
            }
            ServerMessage::DeadMole(dead_id) => {
                for (e, t, v) in moles.iter() {
                    if v.id == dead_id {
                        dbg!("dead mole: {dead_id} ");
                        commands.entity(e).despawn();
                        break;
                    }
                }
            }
            ServerMessage::EscapedMole(_) => {
                todo!();
            }
        }
    }
}
