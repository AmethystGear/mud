use lazy_static::lazy_static;
use serde_jacl::{de::from_str, structs::Value};
use std::{collections::HashMap, fs};
use anyhow::Result;
use serde::Deserialize;
use crate::{deser::block::Block};

#[derive(Debug, Deserialize)]
pub struct TerrainParameters {
    pub map_size: u16,
    pub octaves: u8,
    pub blocks: Vec<String>,
    pub heights: Vec<f64>,
}

#[derive(Debug, Deserialize)]
pub struct GameMode {
    pub blocks: String,
    pub entities: String,
    pub items: String,
    pub terrain: String,
    pub dmg: String,
    pub effect: String,
    pub stat: String,
}

impl GameMode {
    pub fn into(&self) -> Result<GameData> {
        Ok(GameData {
            blocks: from_str(&fs::read_to_string(&self.blocks)?)?,
            entities: from_str(&fs::read_to_string(&self.entities)?)?,
            items: from_str(&fs::read_to_string(&self.items)?)?,
            terrain: from_str(&fs::read_to_string(&self.terrain)?)?,
            dmg: from_str(&fs::read_to_string(&self.dmg)?)?,
            effect: from_str(&fs::read_to_string(&self.effect)?)?,
            stat: from_str(&fs::read_to_string(&self.stat)?)?,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct GameData {
    pub blocks: HashMap<String, Block>,
    pub entities: HashMap<String, EntityTemplate>,
    pub items: HashMap<String, Item>,
    pub terrain: TerrainParameters,
    pub dmg: Vec<String>,
    pub effect: HashMap<String, HashMap<String, Value>>,
    pub stat: Vec<String>,
}


lazy_static! {
    pub static ref GAMEDATA: GameData = {
        let m: GameMode = from_str(&fs::read_to_string("gamemode.jacl").unwrap()).unwrap();
        (&m).into().unwrap()
    };
}
