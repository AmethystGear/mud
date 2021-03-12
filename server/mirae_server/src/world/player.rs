use crate::{inventory::Inventory, location::Vector2, world::spawned::{Creature, Located}, deser::stats::Stats};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize, Serialize)]
pub struct PlayerDeser {
    view: u64,
    stats: Stats,
    inventory: HashMap<String, u64>,
    equip: Vec<String>,
    wears: Vec<String>,
}

impl PlayerDeser {
    pub fn create(&self, loc: Vector2, id: u8) -> Player {
        Player {
            view: self.view,
            stats: self.stats.clone(),
            inventory: Inventory(self.inventory.clone()),
            equip: self.equip.clone().into_iter().next(),
            wears: self.wears.clone(),
            id,
            loc,
        }
    }
}

pub struct Player {
    view: u64,
    stats: Stats,
    inventory: Inventory,
    equip: Option<String>,
    wears: Vec<String>,
    loc: Vector2,
    id: u8,
}

impl Player {
    pub fn id(&self) -> u8 {
        self.id
    }

    pub fn stats(&self) -> &Stats {
        &self.stats
    }
}

impl Located for Player {
    fn loc(&self) -> Vector2 {
        self.loc
    }

    fn set_loc(&mut self, loc: Vector2) {
        self.loc = loc;
    }
}

impl Creature for Player {
    
}
