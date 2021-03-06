use bevy::{prelude::*, utils::HashMap};
use example_shared::{AllExistingMoles, PlayerRank, UpdateScores};
use example_shared::{ClientMessage, MoleDef, MoleKind, ServerMessage, SpawnMole};
use litlnet_server_bevy::{MessagesToRead, MessagesToSend, ServerPlugin};
use litlnet_trait::ClientId;
use litlnet_trait::Server;
use litlnet_websocket_server::ComServer;
use rand::thread_rng;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::env;

fn main() {
    App::new().add_plugin(GamePlugin).run();
}
pub struct GamePlugin;

pub struct MoleIds {
    pub next_id: usize,
}
#[derive(Default)]
pub struct PlayersRanking {
    pub ranks: HashMap<String, usize>,
}
#[derive(Default)]
pub struct PlayersNames {
    pub names: HashMap<ClientId, String>,
}

#[derive(Default)]
pub struct Moles {
    pub moles: HashMap<usize, MoleDef>,
}

pub struct SpawnTimer {
    timer: Timer,
}

pub struct ScoreUpdateTimer {
    timer: Timer,
}
pub struct SpawnDef {
    spawn_area_radius: Vec2,
    offset: Vec2,
}

pub struct RandomDeterministic {
    pub random: ChaCha20Rng,
    pub seed: u64,
}

pub struct ConnectionTarget {
    url: String,
}
impl Default for RandomDeterministic {
    fn default() -> Self {
        let seed = thread_rng().gen::<u64>();
        Self {
            random: ChaCha20Rng::seed_from_u64(seed),
            seed,
        }
    }
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ServerPlugin::<ComServer, ServerMessage, ClientMessage>::default());
        app.add_plugins(MinimalPlugins);
        app.insert_resource(RandomDeterministic::default());
        app.insert_resource(MoleIds { next_id: 0 });
        app.insert_resource(Moles::default());
        app.insert_resource(PlayersNames::default());
        app.insert_resource(PlayersRanking::default());
        app.insert_resource(SpawnTimer {
            timer: Timer::from_seconds(2.5f32, true),
        });
        app.insert_resource(ScoreUpdateTimer {
            timer: Timer::from_seconds(2f32, true),
        });
        app.insert_resource(SpawnDef {
            // resolution of client ("optimized" for itch.io embed rendering)
            spawn_area_radius: Vec2::new(300f32, 150f32),
            offset: Vec2::new(100f32, 0f32),
        });
        let port = env::var("PORT").unwrap_or("8083".to_string());
        app.insert_resource(ConnectionTarget {
            url: format!("0.0.0.0:{}", port),
        });
        app.add_system(receive_messages);
        app.add_system(spawn_moles);
        app.add_system(send_scores);
        app.add_system(reconnect);
    }
}

fn reconnect(connection: Res<ConnectionTarget>, mut com: ResMut<Option<ComServer>>) {
    if com.is_none() {
        dbg!("Reconnection");
        if let Ok(new_com) = ComServer::bind(&connection.url) {
            *com = Some(new_com);
        }
    }
}

fn receive_messages(
    com_server: Res<Option<ComServer>>,
    mut player_names: ResMut<PlayersNames>,
    mut ranking: ResMut<PlayersRanking>,
    mut recv: ResMut<MessagesToRead<ClientMessage>>,
    mut send: ResMut<MessagesToSend<ServerMessage>>,
    mut moles: ResMut<Moles>,
) {
    if let Some(com_server) = com_server.as_ref() {
        while let Some((from_client_id, message)) = recv.pop() {
            match message {
                ClientMessage::HitPosition(position) => {
                    dbg!("HitPosition: ", position);
                    // Check for mole
                    let mut mole_to_die = None;
                    for (id, def) in &moles.moles {
                        if def.position.distance(position) < 50f32 {
                            mole_to_die = Some(*id);
                            break;
                        }
                    }
                    if let Some(mole_to_die) = mole_to_die {
                        *ranking
                            .ranks
                            .entry(
                                player_names
                                    .names
                                    .get(&from_client_id)
                                    .unwrap_or(&"Newbie".to_string())
                                    .clone(),
                            )
                            .or_insert(0) += 1;
                        dbg!("dead mole: {}", mole_to_die);
                        moles.moles.remove(&mole_to_die);
                        let message = ServerMessage::DeadMole {
                            mole_id: mole_to_die,
                            player_killer_id: from_client_id.into(),
                        };
                        for send_client_id in com_server.iter() {
                            send.push((*send_client_id, message.clone()));
                        }
                    }
                    // TODO: if none mole to die, lose points ?
                }
                ClientMessage::RequestAllExistingMoles => {
                    dbg!("RequestAllExistingMoles");
                    let message = ServerMessage::AllExistingMoles(AllExistingMoles {
                        local_player_id: from_client_id.into(),
                        moles: moles
                            .moles
                            .iter()
                            .map(|(id, def)| SpawnMole {
                                id: *id,
                                def: def.clone(),
                            })
                            .collect(),
                    });
                    send.push((from_client_id, message.clone()));
                }
                ClientMessage::SetName(name) => {
                    *player_names
                        .names
                        .entry(from_client_id)
                        .or_insert_with(|| name.clone()) = name.clone();
                }
            }
        }
    }
}
fn spawn_moles(
    mut random: ResMut<RandomDeterministic>,
    time: Res<Time>,
    mut timer: ResMut<SpawnTimer>,
    spawn_def: Res<SpawnDef>,
    com_server: Res<Option<ComServer>>,
    mut mole_ids: ResMut<MoleIds>,
    mut moles: ResMut<Moles>,
    mut send: ResMut<MessagesToSend<ServerMessage>>,
) {
    timer.timer.tick(time.delta());
    if !timer.timer.just_finished() {
        return;
    }
    if let Some(com_server) = com_server.as_ref() {
        let def = MoleDef {
            kind: MoleKind::Duration(2f32),
            position: Vec2::new(
                random
                    .random
                    .gen_range(-spawn_def.spawn_area_radius.x..=spawn_def.spawn_area_radius.x),
                random
                    .random
                    .gen_range(-spawn_def.spawn_area_radius.y..=spawn_def.spawn_area_radius.y),
            ) + spawn_def.offset,
        };
        moles.moles.insert(mole_ids.next_id, def.clone());
        let message = ServerMessage::Spawn(SpawnMole {
            id: mole_ids.next_id,
            def,
        });
        dbg!("new mole");
        mole_ids.next_id += 1;
        for send_client_id in com_server.iter() {
            send.push((*send_client_id, message.clone()));
        }
    }
}

fn send_scores(
    time: Res<Time>,
    mut score_to_send_timer: ResMut<ScoreUpdateTimer>,
    com_server: Res<Option<ComServer>>,
    mut send: ResMut<MessagesToSend<ServerMessage>>,
    mut ranking: ResMut<PlayersRanking>,
) {
    if let Some(com_server) = com_server.as_ref() {
        score_to_send_timer.timer.tick(time.delta());
        if !score_to_send_timer.timer.finished() {
            return;
        }
        let ranking = ServerMessage::UpdateScores(UpdateScores {
            best_players: ranking
                .ranks
                .iter()
                .map(|(k, v)| PlayerRank {
                    name: k.clone(),
                    score: *v,
                })
                .collect(),
        });
        for send_client_id in com_server.iter() {
            send.push((*send_client_id, ranking.clone()));
        }
    }
}
