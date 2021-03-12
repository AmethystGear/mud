use serde::Deserialize;
use serde_jacl::structs::Value;
use std::collections::HashMap;
use anyhow::{Result, anyhow};

#[derive(Debug, Deserialize, Clone)]
pub struct Buffs {
    #[serde(default = "default_hmap")]
    defense_buffs: HashMap<String, f64>,
    #[serde(default = "default_hmap")]
    attack_buffs: HashMap<String, f64>,
    #[serde(default = "default_hmap")]
    stat_buffs: HashMap<String, f64>,
}

impl Buffs {
    pub fn new() -> Self {
        Buffs {
            defense_buffs: HashMap::new(),
            attack_buffs: HashMap::new(),
            stat_buffs: HashMap::new(),
        }
    }
}

fn default_hmap() -> HashMap<String, f64> {
    HashMap::new()
}

fn default_bool() -> bool {
    false
}

fn default_u64() -> u64 {
    0
}

fn default_f64() -> f64 {
    0.0
}

fn default_abilities() -> HashMap<String, HashMap<String, Value>> {
    HashMap::new()
}



#[derive(Debug, Deserialize)]
pub struct Item {
    #[serde(default = "default_bool")]
    wearable: bool,
    #[serde(default = "default_bool")]
    equipable: bool,
    #[serde(default = "default_u64")]
    xp: u64,
    #[serde(default = "Buffs::new")]
    buffs: Buffs
}