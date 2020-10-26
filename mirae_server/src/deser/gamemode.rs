use super::{block::Block, entity::EntityTemplate, item::Item};
use anyhow::Result;
use serde::Deserialize;
use serde_jacl::{de::from_str, structs::Value};
use std::{collections::HashMap, fs};

#[derive(Debug, Deserialize)]
pub struct TerrainParameters {
    pub map_size: u16,
    pub octaves: u8,
    pub blocks: Vec<String>,
    pub heights: Vec<f64>,
}

#[derive(Debug, Deserialize)]
pub struct GameMode {
    pub blocks: String,   // HashMap<String, Block>
    pub entities: String, // HashMap<String, EntityTemplate>
    pub items: String,    // HashMap<String, Item>
    pub terrain: String,  // TerrainParameters
    pub dmg: String,      // Vec<String>
    pub effect: String,   // HashMap<String, HashMap<String, Value>>
    pub stat: String,     // Vec<String>
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
