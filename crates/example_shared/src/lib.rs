use bevy::math::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub enum ClientMessage {
    HitPosition(Vec2),
    RequestAllExistingMoles,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum MoleKind {
    HitCount(u32),
    Duration(f32),
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct MoleDef {
    pub kind: MoleKind,
    pub position: Vec2,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct SpawnMole {
    pub id: usize,
    pub def: MoleDef,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum ServerMessage {
    Spawn(SpawnMole),
    DeadMole(usize),
    EscapedMole(usize),
    AllExistingMoles(Vec<SpawnMole>),
}
