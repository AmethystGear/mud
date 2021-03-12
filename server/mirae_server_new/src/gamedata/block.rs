use super::{gamedata::BlockName, serde_defaults::*};
use crate::rgb::RGB;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct LightingDeser {
    intensity: f64,
    falloff: f64,
    max_range: u64,
    color: RGB,
}

impl LightingDeser {
    fn invalid() -> Self {
        Self {
            intensity: -1.0,
            falloff: -1.0,
            max_range: 0,
            color: RGB::new(0, 0, 0),
        }
    }
}

impl Into<Lighting> for LightingDeser {
    fn into(self) -> Lighting {
        Lighting {
            intensity: self.intensity,
            falloff: self.falloff,
            max_range: self.max_range,
            color: self.color,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct BlockDeser {
    color: RGB,
    #[serde(default = "default_f64")]
    mob_spawn_chance: f64,
    #[serde(default = "default_false")]
    solid: bool,
    #[serde(default = "default_false")]
    z_passable: bool,
    #[serde(default = "LightingDeser::invalid")]
    light: LightingDeser,
}

impl BlockDeser {
    pub fn into_block(self, name: BlockName) -> Block {
        Block {
            name,
            color: self.color,
            mob_spawn_chance: self.mob_spawn_chance,
            solid: self.solid,
            light: if self.light.intensity < 0.0
                || self.light.intensity > 1.0
                || self.light.falloff < 0.0
                || self.light.falloff > 1.0
            {
                None
            } else {
                Some(self.light.into())
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Lighting {
    intensity: f64,
    falloff: f64,
    max_range: u64,
    color: RGB,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub name: BlockName,
    pub color: RGB,
    pub mob_spawn_chance: f64,
    pub solid: bool,
    pub light: Option<Lighting>,
}
