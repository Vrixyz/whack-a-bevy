use std::{collections::VecDeque, marker::PhantomData};

use bevy::prelude::*;
use litlnet_trait::{ClientId, Server};
use serde::{de::DeserializeOwned, Serialize};

pub struct ServerPlugin<C: Server, S: Serialize, R: DeserializeOwned> {
    _phantom_c: Option<PhantomData<C>>,
    _phantom_s: Option<PhantomData<S>>,
    _phantom_r: Option<PhantomData<R>>,
}
impl<C: Server, S: Serialize, R: DeserializeOwned> Default for ServerPlugin<C, S, R> {
    fn default() -> Self {
        Self {
            _phantom_c: None,
            _phantom_s: None,
            _phantom_r: None,
        }
    }
}
pub struct MessagesToSend<S: Serialize> {
    messages: VecDeque<(ClientId, S)>,
}

impl<S: Serialize> Default for MessagesToSend<S> {
    fn default() -> Self {
        Self {
            messages: VecDeque::new(),
        }
    }
}
impl<S: Serialize> MessagesToSend<S> {
    pub fn push(&mut self, message: (ClientId, S)) {
        self.messages.push_back(message);
    }
}

pub struct MessagesToRead<R: DeserializeOwned> {
    messages: VecDeque<(ClientId, R)>,
}
impl<R: DeserializeOwned> Default for MessagesToRead<R> {
    fn default() -> Self {
        Self {
            messages: VecDeque::new(),
        }
    }
}
impl<R: DeserializeOwned> MessagesToRead<R> {
    pub fn pop(&mut self) -> Option<(ClientId, R)> {
        self.messages.pop_front()
    }
}
impl<
        C: Server + Send + Sync + 'static,
        S: Serialize + Send + Sync + 'static,
        R: DeserializeOwned + Send + Sync + 'static,
    > Plugin for ServerPlugin<C, S, R>
{
    fn build(&self, app: &mut App) {
        let com: Option<C> = None;
        app.insert_resource(com);
        app.insert_resource(MessagesToRead::<R>::default());
        app.insert_resource(MessagesToSend::<S>::default());
        app.add_system(accept_connections::<C>);
        app.add_system(receive_messages::<C, R>);
        app.add_system(send_messages::<C, S>);
    }
}
fn accept_connections<C: Server + Send + Sync + 'static>(mut com_to_read: ResMut<Option<C>>) {
    if let Some(com_to_read) = com_to_read.as_mut() {
        com_to_read.accept_connections();
    }
}
fn receive_messages<
    C: Server + Send + Sync + 'static,
    R: DeserializeOwned + Send + Sync + 'static,
>(
    mut com: ResMut<Option<C>>,
    mut messages_to_read: ResMut<MessagesToRead<R>>,
) {
    if let Some(com) = com.as_mut() {
        com.receive_all(|id, messages| {
            for message in messages {
                messages_to_read.messages.push_back((id, message));
            }
        });
    }
}

fn send_messages<C: Server + Send + Sync + 'static, S: Serialize + Send + Sync + 'static>(
    mut com: ResMut<Option<C>>,
    mut messages_to_send: ResMut<MessagesToSend<S>>,
) {
    if let Some(com) = com.as_mut() {
        for msg in messages_to_send.messages.iter() {
            com.send(&msg.0, &msg.1);
        }
        messages_to_send.messages.clear();
    }
}
