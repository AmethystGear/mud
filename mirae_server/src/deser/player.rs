use super::stats::Stats;
use serde::Deserialize;
use std::collections::HashMap;
use crate::{location::Vector2, inventory::Inventory};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize)]
pub struct PlayerDeser {
    view : u64,
    stats: Stats,
    inventory: HashMap<String, u64>,
    equips: Vec<String>,
    wears: Vec<String>
}

pub struct Player {
    view : u64,
    stats : Stats,
    inventory: Inventory,
    equips: Vec<String>,
    wears: Vec<String>,
    loc : Vector2,
    cumulative_speed: f64,
    turn: bool,
    last_turn: SystemTime,
    id : u8
}

impl Player {
    pub fn loc(&self) -> Vector2 {
        self.loc
    }

    pub fn id(&self) -> u8 {
        self.id
    }
}