use bevy::prelude::*;
use example_shared::{ClientMessage, ServerMessage, Spawn};
use litlnet_server_bevy::{MessagesToRead, MessagesToSend, ServerPlugin};
use litlnet_trait::Server;
use litlnet_websocket_server::ComServer;

fn main() {
    App::new().add_plugin(GamePlugin).run();
}
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ServerPlugin::<ComServer, ServerMessage, ClientMessage>::default())
            .add_plugins(MinimalPlugins);
        app.add_system(rebroadcast_messages);
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

fn rebroadcast_messages(
    com_server: Res<Option<ComServer>>,
    mut recv: ResMut<MessagesToRead<ClientMessage>>,
    mut send: ResMut<MessagesToSend<ServerMessage>>,
) {
    if let Some(com_server) = com_server.as_ref() {
        while let Some((from_client_id, message)) = recv.pop() {
            let ClientMessage::Position(position) = message;
            let message = ServerMessage::Spawn(Spawn {
                client_id: from_client_id.into(),
                position,
            });
            for send_client_id in com_server.iter() {
                send.push((*send_client_id, message.clone()));
            }
        }
    }
}
