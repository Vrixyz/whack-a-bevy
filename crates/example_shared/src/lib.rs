use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize, Debug, PartialEq, Default, Clone)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub enum ClientMessage {
    Position(Vec2),
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct Spawn {
    pub client_id: usize,
    pub position: Vec2,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum ServerMessage {
    Spawn(Spawn),
}
