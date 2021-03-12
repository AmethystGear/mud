use super::spawned::{Handle, Located};
use std::{collections::HashMap, time::{SystemTime, UNIX_EPOCH}};
use crate::deser::stats::Stats;
use serde_jacl::structs::Value;

pub trait GameObject : Fighter + Located {

}

pub trait Fighter {
    fn stats(&self) -> &Stats;
    fn curr_stats(&self) -> &Stats;
}

struct FighterData {
    handle: Handle,
    cumulative_speed: u64,
    turn_time: SystemTime,
    effects: Vec<HashMap<String, Value>>
}

impl FighterData {
    pub fn new(handle: Handle) -> FighterData {
        FighterData {
            handle,
            cumulative_speed: 0,
            turn_time: UNIX_EPOCH,
            effects: Vec::new()
        }
    }
}

struct Battle {
    attacker : FighterData,
    defender : FighterData,
}

impl Battle {
    pub fn new(attacker : Handle, defender : Handle) -> Battle {
        Battle {
            attacker: FighterData::new(attacker),
            defender: FighterData::new(defender)
        }
    }
}