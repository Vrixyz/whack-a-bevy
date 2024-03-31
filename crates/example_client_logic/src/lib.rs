mod cheatbook;

use bevy::{
    math::Vec3Swizzles,
    prelude::*,
    window::{PrimaryWindow, WindowResolution},
};

use example_shared::{AllExistingMoles, ClientMessage, ServerMessage};
use litlnet_client_bevy::{ClientPlugin, MessagesToRead, MessagesToSend, RComClient};

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

#[derive(Resource)]
struct AssetsVisualPlayer {
    pub sprite_handles: Vec<Handle<Image>>,
}
#[derive(Default, Resource)]
struct AssetsExplosions {
    pub explosion_local: Vec<Handle<Image>>,
    pub explosion_remote: Vec<Handle<Image>>,
}

#[derive(Component)]
struct ExplosionData {
    timer: Timer,
    sprite_index: usize,
    kind: ExplosionKind,
}

#[derive(Resource)]
enum WantToRequestExisting {
    No,
    Yes(Timer),
}

#[derive(Resource)]
pub struct ReconnectState {
    timer: Timer,
    attempt: usize,
}

#[derive(PartialEq, Clone)]
enum ExplosionKind {
    LocalPlayer,
    RemotePlayer,
}

#[derive(Event)]
pub struct SpawnExplosionEvent {
    position: Vec2,
    kind: ExplosionKind,
}

#[derive(Resource)]
pub struct LocalPlayer {
    // used to map score (we can have multiple players bringing score to the same rank,
    // it's ok because I won't dev a full authentication system for a jam yet.)
    name: String,
    is_final: bool,
    // used to know who killed which mole
    id: Option<usize>,
    score: Option<u32>,
}

#[derive(Resource)]
pub struct RemotePlayers {
    players: Vec<(String, u32)>,
}

mod ui {
    use bevy::{prelude::*, reflect::List};
    use bevy_egui::{EguiContext, EguiContexts, EguiPlugin};
    use egui::{Color32, RichText, Vec2};
    use example_shared::ClientMessage;
    use litlnet_client_bevy::{MessagesToSend, RComClient};

    use crate::{ComClient, LocalPlayer, ReconnectState, RemotePlayers, VisualMole};
    pub struct GameUI;

    impl Plugin for GameUI {
        fn build(&self, app: &mut App) {
            app.add_plugins(EguiPlugin);
            app.insert_resource(LocalPlayer {
                name: "Newbie".to_string(),
                is_final: false,
                id: None,
                score: None,
            });
            app.insert_resource(RemotePlayers { players: vec![] });
            app.add_systems(Update, show_name);
            app.add_systems(Update, display_connection);
        }
    }

    fn show_name(
        mut send: ResMut<MessagesToSend<ClientMessage>>,
        mut contexts: EguiContexts,
        mut local_player: ResMut<LocalPlayer>,
        remote_players: Res<RemotePlayers>,
        moles: Query<Entity, With<VisualMole>>,
    ) {
        egui::Window::new("Info")
            .fixed_size(Vec2::new(150f32, 400f32))
            .anchor(egui::Align2::LEFT_TOP, Vec2::default())
            .show(contexts.ctx_mut(), |ui| {
                if (local_player.is_final) {
                    ui.label(format!("Nickname: {}", local_player.name));
                } else {
                    ui.text_edit_singleline(&mut local_player.name);
                    local_player.name = local_player.name.chars().take(6).collect::<String>();
                    let is_send_name_clicked =
                        if !local_player.is_final && local_player.name != "Newbie" {
                            ui.button(RichText::new("send nickname").color(Color32::LIGHT_GREEN))
                                .clicked()
                        } else {
                            ui.button("send nickname").clicked()
                        };
                    if is_send_name_clicked {
                        dbg!("send name={}", &local_player.name);
                        send.push(ClientMessage::SetName(local_player.name.clone()));
                        local_player.is_final = true;
                    }
                }
                ui.label("");
                ui.label(format!(
                    "Number of bevies to whack: {}",
                    moles.iter().count()
                ));
                ui.label("");
                ui.label("SCORES:");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut current_player_found = false;
                    for (name, score) in &remote_players.players {
                        if !current_player_found && name == &local_player.name {
                            ui.colored_label(Color32::LIGHT_GREEN, format!("{}: {}", name, score));
                            current_player_found = true;
                        } else {
                            ui.label(format!("{}: {}", name, score));
                        }
                    }
                });
            });
    }
    fn display_connection(
        mut contexts: EguiContexts,
        client: Option<Res<RComClient<ComClient>>>,
        reconnect_state: Res<ReconnectState>,
    ) {
        if client.is_some() {
            return;
        }
        egui::Window::new("Connection to server...")
            .anchor(egui::Align2::CENTER_CENTER, Vec2::default())
            .show(contexts.ctx_mut(), |ui| {
                ui.label(format!(
                    "retrying in {} seconds",
                    (reconnect_state.timer.duration().as_secs_f32()
                        - reconnect_state.timer.elapsed_secs()) as i32
                ));
                match reconnect_state.attempt {
                    0 => {}
                    1..=2 => {
                        ui.label("Connecting to server...");
                    },
                    3..=4 => {
                        ui.label("Server restarts after 30 minutes of inactivity, so chances are you're the first one to connect! Hang thight.");
                    },
                    5..=6 => {
                        ui.label("I'm using Heroku Free, that's why there's the restart...");
                    },
                    7 => {
                        ui.label("As the first player, you'll have the UNFAIR ADVANTAGE to be able to hit more bevies!");
                    },
                    8 => {
                        ui.label("If you feel like you're addicted to video games, you might want to seek help from professionnals.");
                    },
                    9 => {
                        ui.label("Ok it's definitely taking more time than necessary, please tell me :).");
                    },
                    10 => {
                        ui.label("do you think there's more messages..?");
                    }
                    11.. => {
                        ui.label("Alright there was one other message, thanks for really giving this a try, now please drop me a message and try at another time <3");
                    }
                    _ => {
                        ui.label("unexpected attempt value, how much time have you waited ? :o");
                    }
                }
            });
    }
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Whack A Bevy!".to_string(),
                resolution: WindowResolution::new(1080., 640.),
                ..Default::default()
            }),
            ..default()
        }));
        app.add_plugins(ClientPlugin::<
            RComClient<ComClient>,
            ClientMessage,
            ServerMessage,
        >::default());
        app.add_plugins(ui::GameUI);
        app.insert_resource(WantToRequestExisting::No);
        app.insert_resource(ReconnectState {
            timer: Timer::from_seconds(0.01f32, TimerMode::Repeating),
            attempt: 0,
        });
        app.insert_resource(AssetsExplosions::default());
        app.add_event::<SpawnExplosionEvent>();
        app.add_systems(Startup, setup);
        app.add_systems(Update, reconnect);
        app.add_systems(Update, check_request_existing);
        app.add_systems(Update, send_messages);
        app.add_systems(Update, receive_messages);
        app.add_systems(Update, spawn_explosions);
        app.add_systems(Update, explosion_lifecycle);
    }
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut explosions: ResMut<AssetsExplosions>,
) {
    commands.spawn(Camera2dBundle::default()).insert(MainCamera);
    let sprite_handle = assets.load("players/icon_bevy.png");
    commands.insert_resource(AssetsVisualPlayer {
        sprite_handles: vec![sprite_handle],
    });
    explosions.explosion_local = vec![
        assets.load("explosions/local_1.png"),
        assets.load("explosions/local_2.png"),
        assets.load("explosions/local_3.png"),
    ];
    explosions.explosion_remote = vec![
        assets.load("explosions/remote_1.png"),
        assets.load("explosions/remote_2.png"),
        assets.load("explosions/remote_3.png"),
    ];
}

fn reconnect(
    time: Res<Time>,
    mut commands: Commands,
    client: Option<ResMut<RComClient<ComClient>>>,
    mut want_request_existing: ResMut<WantToRequestExisting>,
    mut reconnect_state: ResMut<ReconnectState>,
) {
    if client.is_some() {
        return;
    }
    reconnect_state.timer.tick(time.delta());
    if !reconnect_state.timer.just_finished() {
        return;
    }
    let current_duration = reconnect_state.timer.duration().as_secs_f32();
    reconnect_state
        .timer
        .set_duration(std::time::Duration::from_secs_f32(current_duration + 2f32));
    #[cfg(target_arch = "wasm32")]
    let server_url = option_env!("WEB_SERVER_URL").unwrap_or("ws://127.0.0.1:8083");
    #[cfg(not(target_arch = "wasm32"))]
    let server_url =
        &std::env::var("WEB_SERVER_URL").unwrap_or_else(|_| "ws://127.0.0.1:8083".to_string());
    reconnect_state.attempt += 1;
    if let Ok(ws) = ComClient::connect(server_url) {
        commands.insert_resource(RComClient { com: ws });
        reconnect_state.attempt = 0;
        reconnect_state
            .timer
            .set_duration(std::time::Duration::from_secs_f32(2f32));
        *want_request_existing =
            WantToRequestExisting::Yes(Timer::from_seconds(0.5f32, TimerMode::Once))
    }
}

fn check_request_existing(
    time: Res<Time>,
    mut send: ResMut<MessagesToSend<ClientMessage>>,
    mut want_request_existing: ResMut<WantToRequestExisting>,
) {
    match want_request_existing.as_mut() {
        WantToRequestExisting::Yes(ref mut timer) => {
            timer.tick(time.delta());
            if timer.just_finished() {
                send.push(ClientMessage::RequestAllExistingMoles);
                timer.set_duration(std::time::Duration::from_secs_f32(2.5f32));
                timer.reset();
            }
        }
        _ => (),
    }
}

fn send_messages(
    mut send: ResMut<MessagesToSend<ClientMessage>>,
    buttons: Res<ButtonInput<MouseButton>>,
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        // get the camera info and transform
        // assuming there is exactly one main camera entity, so Query::single() is OK
        let (camera, camera_transform) = q_camera.single();

        // There is only one primary window, so we can similarly get it from the query:
        let window = q_window.single();
        if let Some(world_position) = cheatbook::cursor_to_world(window, q_camera.single()) {
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
    mut spawn_explosions_events: EventWriter<SpawnExplosionEvent>,
    mut want_request_existing: ResMut<WantToRequestExisting>,
    mut recv: ResMut<MessagesToRead<ServerMessage>>,
    mut moles: Query<(Entity, &Transform, &VisualMole)>,
    mut local_player: ResMut<LocalPlayer>,
    mut rankings: ResMut<RemotePlayers>,
) {
    while let Some(message) = recv.pop() {
        match message {
            ServerMessage::Spawn(spawn) => {
                dbg!("new mole: {}", &spawn);
                spawn_mole(&mut commands, &sprites, spawn);
            }
            ServerMessage::DeadMole {
                mole_id: dead_id,
                player_killer_id,
            } => {
                dbg!("?dead mole: {}", dead_id);
                for (e, t, v) in moles.iter() {
                    if v.id == dead_id {
                        if local_player.id.is_some() && local_player.id.unwrap() == player_killer_id
                        {
                            spawn_explosions_events.send(SpawnExplosionEvent {
                                kind: ExplosionKind::LocalPlayer,
                                position: t.translation.xy(),
                            });
                        } else {
                            spawn_explosions_events.send(SpawnExplosionEvent {
                                kind: ExplosionKind::RemotePlayer,
                                position: t.translation.xy(),
                            });
                        }
                        dbg!("dead mole: {}", dead_id);
                        commands.entity(e).despawn();
                        break;
                    }
                }
            }
            ServerMessage::EscapedMole(_) => {
                todo!("escaped mole");
            }
            ServerMessage::AllExistingMoles(AllExistingMoles {
                local_player_id,
                moles: existing_moles,
            }) => {
                for (e, _, _) in moles.iter() {
                    commands.entity(e).despawn();
                }
                local_player.id = Some(local_player_id);
                dbg!("{} existing moles", existing_moles.len());
                for m in existing_moles {
                    spawn_mole(&mut commands, &sprites, m);
                }
                *want_request_existing = WantToRequestExisting::No;
            }
            ServerMessage::UpdateScores(update) => {
                rankings.players = update
                    .best_players
                    .iter()
                    .map(|v| (v.name.clone(), v.score as u32))
                    .collect();
                rankings
                    .players
                    .sort_by(|p1, p2| p2.1.partial_cmp(&p1.1).unwrap());
            }
        }
    }
}

fn spawn_mole(
    commands: &mut Commands,
    sprites: &Res<AssetsVisualPlayer>,
    spawn: example_shared::SpawnMole,
) {
    commands
        .spawn(SpriteBundle {
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

fn spawn_explosions(
    mut commands: Commands,
    mut events: EventReader<SpawnExplosionEvent>,
    sprites: Res<AssetsExplosions>,
) {
    for mut event in events.read() {
        dbg!("spawn explosion!!!!");
        commands
            .spawn(ExplosionData {
                sprite_index: 0,
                timer: Timer::from_seconds(0.12, TimerMode::Repeating),
                kind: event.kind.clone(),
            })
            .insert(SpriteBundle {
                texture: if event.kind == ExplosionKind::LocalPlayer {
                    sprites.explosion_local[0].clone()
                } else {
                    sprites.explosion_remote[0].clone()
                },
                transform: Transform::from_translation(event.position.extend(10f32)),
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(64.0)),
                    ..Default::default()
                },
                ..Default::default()
            });
    }
}

fn explosion_lifecycle(
    mut commands: Commands,
    time: Res<Time>,
    sprites: Res<AssetsExplosions>,
    mut q_explosions: Query<(Entity, &mut ExplosionData, &mut Handle<Image>)>,
) {
    for (e, mut explosion, mut sprite) in q_explosions.iter_mut() {
        explosion.timer.tick(time.delta());
        if explosion.timer.just_finished() {
            explosion.sprite_index += 1;
            if explosion.kind == ExplosionKind::LocalPlayer
                && explosion.sprite_index >= sprites.explosion_local.len()
            {
                commands.entity(e).despawn();
                continue;
            }
            if explosion.kind == ExplosionKind::RemotePlayer
                && explosion.sprite_index >= sprites.explosion_remote.len()
            {
                commands.entity(e).despawn();
                continue;
            }
            *sprite = if explosion.kind == ExplosionKind::RemotePlayer {
                sprites.explosion_remote[explosion.sprite_index].clone()
            } else {
                sprites.explosion_local[explosion.sprite_index].clone()
            };
        }
    }
}
