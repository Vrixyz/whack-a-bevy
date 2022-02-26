use bevy::{prelude::*, utils::HashMap};
use example_shared::{ClientMessage, MoleDef, MoleKind, ServerMessage, SpawnMole};
use litlnet_server_bevy::{MessagesToRead, MessagesToSend, ServerPlugin};
use litlnet_trait::Server;
use litlnet_websocket_server::ComServer;
use rand::SeedableRng;
use rand::thread_rng;
use rand::Rng;
use rand_chacha::ChaCha20Rng;

fn main() {
    App::new().add_plugin(GamePlugin).run();
}
pub struct GamePlugin;

pub struct MoleIds {
    pub next_id: usize,
}

#[derive(Default)]
pub struct Moles {
    pub moles: HashMap<usize, MoleDef>,
}

pub struct SpawnTimer {
    timer: Timer,
}

pub struct SpawnDef {
    spawn_area_radius: Vec2,
}

pub struct RandomDeterministic {
    pub random: ChaCha20Rng,
    pub seed: u64,
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
        app.insert_resource(SpawnTimer {
            timer: Timer::from_seconds(2.5f32, true),
        });
        app.insert_resource(SpawnDef {
            spawn_area_radius: Vec2::new(400f32, 200f32),
        });
        app.add_system(receive_messages);
        app.add_system(spawn_moles);
        app.add_system(reconnect);
    }
}

fn reconnect(mut com: ResMut<Option<ComServer>>) {
    if com.is_none() {
        if let Ok(new_com) = ComServer::bind("127.0.0.1:8083") {
            *com = Some(new_com);
        }
    }
}

fn receive_messages(
    com_server: Res<Option<ComServer>>,
    mut recv: ResMut<MessagesToRead<ClientMessage>>,
    mut send: ResMut<MessagesToSend<ServerMessage>>,
    mut moles: ResMut<Moles>,
) {
    if let Some(com_server) = com_server.as_ref() {
        while let Some((from_client_id, message)) = recv.pop() {
            let ClientMessage::HitPosition(position) = message;
            // Check for mole
            let mut mole_to_die = None;
            for (id, def) in &moles.moles {
                if def.position.distance(position) < 50f32 {
                    mole_to_die = Some(*id);
                }
            }
            if let Some(mole_to_die) = mole_to_die {
                dbg!("dead mole: {mole_to_die} ");
                moles.moles.remove(&mole_to_die);
                let message = ServerMessage::DeadMole(mole_to_die);
                for send_client_id in com_server.iter() {
                    send.push((*send_client_id, message.clone()));
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
    if !timer.timer.finished() {
        return;
    }
    timer.timer.reset();
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
            ),
        };
        moles.moles.insert(mole_ids.next_id, def.clone());
        let message = ServerMessage::Spawn(SpawnMole {
            id: mole_ids.next_id,
            def,
        });
        mole_ids.next_id += 1;
        for send_client_id in com_server.iter() {
            send.push((*send_client_id, message.clone()));
        }
    }
}
