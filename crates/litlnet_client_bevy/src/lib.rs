use std::{collections::VecDeque, marker::PhantomData};

use bevy::prelude::*;
use litlnet_trait::Communication;
use serde::{de::DeserializeOwned, Serialize};

pub struct ClientPlugin<C: Communication, S: Serialize, R: DeserializeOwned> {
    _phantom_c: Option<PhantomData<C>>,
    _phantom_s: Option<PhantomData<S>>,
    _phantom_r: Option<PhantomData<R>>,
}

impl<C: Communication, S: Serialize, R: DeserializeOwned> Default for ClientPlugin<C, S, R> {
    fn default() -> Self {
        Self {
            _phantom_c: None,
            _phantom_s: None,
            _phantom_r: None,
        }
    }
}

#[derive(Resource)]
pub struct RComClient<C: Communication + Send + Sync + 'static> {
    pub com: C,
}

impl<C> Communication for RComClient<C>
where
    C: Communication + Send + Sync + 'static,
{
    fn receive<T: DeserializeOwned>(&mut self) -> Result<Option<Vec<T>>, std::io::Error> {
        self.com.receive()
    }

    fn send<T: Serialize>(&mut self, message: &T) -> std::io::Result<()> {
        self.com.send(message)
    }
}

#[derive(Resource)]
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

#[derive(Resource)]
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
impl<C, S, R> Plugin for ClientPlugin<C, S, R>
where
    C: Resource + Communication + Send + Sync + 'static,
    S: Serialize + Send + Sync + 'static,
    R: DeserializeOwned + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(MessagesToRead::<R>::default());
        app.insert_resource(MessagesToSend::<S>::default());
        app.add_systems(Update, receive_messages::<C, R>);
        app.add_systems(Update, send_messages::<C, S>);
    }
}
fn receive_messages<
    C: Resource + Communication + Send + Sync + 'static,
    R: DeserializeOwned + Send + Sync + 'static,
>(
    mut com: Option<ResMut<C>>,
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

fn send_messages<
    C: Resource + Communication + Send + Sync + 'static,
    S: Serialize + Send + Sync + 'static,
>(
    mut commands: Commands,
    mut com: Option<ResMut<C>>,
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
        commands.remove_resource::<C>();
    }
}
