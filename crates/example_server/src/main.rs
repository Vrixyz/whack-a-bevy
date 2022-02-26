use bevy::{prelude::*, utils::HashMap};
use example_shared::{ClientMessage, MoleDef, MoleKind, ServerMessage, SpawnMole};
use litlnet_server_bevy::{MessagesToRead, MessagesToSend, ServerPlugin};
use litlnet_trait::Server;
use litlnet_websocket_server::ComServer;

fn main() {
    App::new().add_plugin(GamePlugin).run();
}
pub struct GamePlugin;

pub struct MoleIds {
    pub next_id: u32,
}
pub struct Moles {
    pub moles: HashMap<u32, MoleDef>,
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ServerPlugin::<ComServer, ServerMessage, ClientMessage>::default())
            .add_plugins(MinimalPlugins);
        app.insert_resource(MoleIds { next_id: 0 });
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
                if Vec2::from(def.position).distance(position) < 50f32 {
                    mole_to_die = Some(*id);
                }
            }
            if let Some(mole_to_die) = mole_to_die {
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
    com_server: Res<Option<ComServer>>,
    mut mole_ids: ResMut<MoleIds>,
    mut moles: ResMut<Moles>,
    mut send: ResMut<MessagesToSend<ServerMessage>>,
) {
    // TODO: put a timer
    if let Some(com_server) = com_server.as_ref() {
        let def = MoleDef {
            kind: MoleKind::Duration(2f32),
            position: Vec2::new(0f32, 0f32),
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
