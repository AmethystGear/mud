use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::collections::HashMap;
pub struct Blocks(HashMap<String, Block>);
pub struct BlockName {
    name: String,
}
impl BlockName {
    pub fn new<S: Into<String>>(name: S, blocks: &Blocks) -> Result<Self> {
        let name = name.into();
        if blocks.0.contains_key(&name) {
            Ok(BlockName { name })
        } else {
            Err(anyhow!("block name {} not in blocks {:?}", name, blocks.0))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Block {
    pub display: String,
    #[serde(default = "default_mob_spawn_chance")]
    pub mob_spawn_chance: f64,
    #[serde(default = "default_solid")]
    pub solid: bool,
    #[serde(default = "default_mob_filter")]
    pub mob_filter: Vec<String>,
}

fn default_mob_spawn_chance() -> f64 {
    0.0
}

fn default_solid() -> bool {
    false
}

fn default_mob_filter() -> Vec<String> {
    vec![]
}
