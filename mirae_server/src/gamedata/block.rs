use super::{gamedata::BlockName, serde_defaults::*};
use crate::rgb::RGB;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct PointLight {
    pub intensity: f64,
    pub falloff: f64,
    pub max_range: u64,
    pub color: RGB,
}

impl PointLight {
    fn invalid() -> Self {
        Self {
            intensity: -1.0,
            falloff: -1.0,
            max_range: 0,
            color: RGB::new(0, 0, 0),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct DownLight {
    pub intensity: f64,
    pub color: RGB,
}

impl DownLight {
    fn invalid() -> Self {
        Self {
            intensity: -1.0,
            color: RGB::new(0, 0, 0),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct LightingDeser {
    #[serde(default = "PointLight::invalid")]
    point_light: PointLight,
    #[serde(default = "DownLight::invalid")]
    down_light: DownLight,
}

impl LightingDeser {
    fn invalid() -> Self {
        Self {
            point_light: PointLight::invalid(),
            down_light: DownLight::invalid(),
        }
    }
}

impl Into<Lighting> for LightingDeser {
    fn into(self) -> Lighting {
        Lighting {
            point_light: if self.point_light.intensity < 0.0 {
                None
            } else {
                Some(self.point_light)
            },
            down_light : if self.down_light.intensity < 0.0 {
                None
            } else {
                Some(self.down_light)
            }
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct BlockDeser {
    color: RGB,
    #[serde(default = "zero_f64")]
    mob_spawn_chance: f64,
    #[serde(default = "false_bool")]
    solid: bool,
    #[serde(default = "false_bool")]
    z_passable: bool,
    #[serde(default = "false_bool")]
    unlit: bool,
    #[serde(default = "LightingDeser::invalid")]
    light: LightingDeser,
    #[serde(default = "RGB::black")]
    transparency: RGB,
}

impl BlockDeser {
    pub fn into_block(self, name: BlockName) -> Block {
        Block {
            name,
            color: self.color,
            mob_spawn_chance: self.mob_spawn_chance,
            solid: self.solid,
            light: self.light.into(),
            transparency: self.transparency,
            unlit: self.unlit,
            z_passable : self.z_passable
        }
    }
}

#[derive(Debug, Clone)]
pub struct Lighting {
    pub point_light: Option<PointLight>,
    pub down_light: Option<DownLight>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub name: BlockName,
    pub color: RGB,
    pub mob_spawn_chance: f64,
    pub solid: bool,
    pub z_passable : bool,
    pub light: Lighting,
    pub unlit: bool,
    pub transparency: RGB,
}
