use super::{
    gamedata::{BlockName, ItemName},
    serde_defaults::*,
};
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
            down_light: if self.down_light.intensity < 0.0 {
                None
            } else {
                Some(self.down_light)
            },
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct BlockDeser {
    #[serde(default = "RGB::black")]
    color: RGB,
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
    #[serde(default = "MobInfo::new")]
    mob_spawn: MobInfo,
    #[serde(default = "empty_string")]
    break_into: String,
    #[serde(default = "empty_string")]
    drop: String,
    #[serde(default = "empty_string")]
    texture: String,
}

impl BlockDeser {
    pub fn into_block(self, name: BlockName) -> Block {
        let break_into = if self.break_into == "" {
            None
        } else {
            Some(BlockName::from(self.break_into))
        };
        let drop = if self.drop == "" {
            None
        } else {
            Some(ItemName::from(self.drop))
        };
        Block {
            name,
            color: self.color,
            solid: self.solid,
            light: self.light.into(),
            transparency: self.transparency,
            unlit: self.unlit,
            z_passable: self.z_passable,
            mob_spawn: self.mob_spawn,
            break_into,
            drop,
            texture: if self.texture == "" {
                None
            } else {
                Some(self.texture)
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Lighting {
    pub point_light: Option<PointLight>,
    pub down_light: Option<DownLight>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MobInfo {
    #[serde(default = "empty_vec")]
    pub exclude: Vec<String>,
    #[serde(default = "empty_vec")]
    pub require: Vec<String>,
    #[serde(default = "empty_vec")]
    pub favor: Vec<String>,
    #[serde(default = "zero_f64")]
    pub favor_prob: f64,
    #[serde(default = "zero_f64")]
    pub spawn_chance: f64,
}

impl MobInfo {
    pub fn new() -> MobInfo {
        MobInfo {
            exclude: vec![],
            require: vec![],
            favor: vec![],
            favor_prob: 0.0,
            spawn_chance: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub name: BlockName,
    pub color: RGB,
    pub solid: bool,
    pub z_passable: bool,
    pub light: Lighting,
    pub unlit: bool,
    pub transparency: RGB,
    pub mob_spawn: MobInfo,
    pub break_into: Option<BlockName>,
    pub drop: Option<ItemName>,
    pub texture: Option<String>,
}
