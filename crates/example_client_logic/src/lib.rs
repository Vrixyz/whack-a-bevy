mod cheatbook;

use bevy::prelude::*;

use example_shared::{ClientMessage, ServerMessage};
use litlnet_client_bevy::{ClientPlugin, MessagesToRead, MessagesToSend};

#[cfg(target_arch = "wasm32")]
type ComClient = litlnet_websocket_web::WebsocketClient;

#[cfg(not(target_arch = "wasm32"))]
type ComClient = litlnet_websocket::WebsocketClient;

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
        app.add_plugin(ClientPlugin::<ComClient, ClientMessage, ServerMessage>::default());
        app.add_startup_system(setup);
        app.add_system(send_messages);
        app.add_system(receive_messages);
    }
}

pub fn setup(
    mut commands: Commands,
    mut send: ResMut<MessagesToSend<ClientMessage>>,
    assets: Res<AssetServer>,
    mut client: ResMut<Option<ComClient>>,
) {
    #[cfg(target_arch = "wasm32")]
    let server_url = option_env!("WAB_SERVER_URL").unwrap_or("ws://127.0.0.1:8083");
    #[cfg(not(target_arch = "wasm32"))]
    let server_url =
        &std::env::var("WAB_SERVER_URL").unwrap_or_else(|_| "ws://127.0.0.1:8083".to_string());
    if let Ok(ws) = ComClient::connect(server_url) {
        *client = Some(ws);
        send.push(ClientMessage::RequestAllExistingMoles);
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
                spawn_mole(&mut commands, &sprites, spawn);
            }
            ServerMessage::DeadMole(dead_id) => {
                dbg!("?dead mole: {}", dead_id);
                for (e, t, v) in moles.iter() {
                    if v.id == dead_id {
                        dbg!("dead mole: {}", dead_id);
                        commands.entity(e).despawn();
                        break;
                    }
                }
            }
            ServerMessage::EscapedMole(_) => {
                todo!();
            }
            ServerMessage::AllExistingMoles(moles) => {
                for m in moles {
                    spawn_mole(&mut commands, &sprites, m);
                }
            }
        }
    }
}

fn spawn_mole(
    commands: &mut Commands,
    sprites: &Res<AssetsVisualPlayer>,
    spawn: example_shared::SpawnMole,
) {
    dbg!("new mole: {}", &spawn);
    commands
        .spawn()
        .insert_bundle(SpriteBundle {
            texture: sprites.sprite_handles[0].clone(),
            transform: Transform::from_translation(spawn.def.position.extend(0f32)),
            sprite: Sprite {
                custom_size: Some(Vec2::splat(64.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(VisualMole { id: spawn.id });
}
