use super::{
    gamedata::{DmgType, GameData, ItemName, StatType},
    serde_defaults::*,
};
use crate::stat::default_empty_fields;
use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[derive(Deserialize, Debug)]
pub struct AbilityDeser {
    #[serde(default = "false_bool")]
    destroy_item: bool,
    #[serde(default = "zero_u64")]
    stun: u64,
    #[serde(default = "zero_u64")]
    charge: u64,
    #[serde(default = "zero_u64")]
    repeat: u64,
    #[serde(default = "zero_f64")]
    health: f64,
    #[serde(default = "zero_f64")]
    energy: f64,

    #[serde(default = "empty_hmap")]
    damage: HashMap<String, f64>,
    #[serde(default = "empty_hmap")]
    block: HashMap<String, f64>,
    #[serde(default = "empty_hmap")]
    counter: HashMap<String, f64>,

    #[serde(default = "empty_hmap")]
    require_items: HashMap<String, u64>,
    #[serde(default = "empty_hmap")]
    remove_items: HashMap<String, u64>,
    #[serde(default = "empty_hmap")]
    make_items: HashMap<String, u64>,

    #[serde(default = "zero_i64")]
    xp_cost: i64,

    #[serde(default = "one_f64")]
    accuracy: f64,

    #[serde(default = "empty_string")]
    text: String,

    #[serde(default = "empty_string")]
    self_text: String,
}

impl AbilityDeser {
    pub fn into_ability(
        self,
        name: String,
        dmg_types: &HashSet<DmgType>,
        item_names: &HashSet<ItemName>,
    ) -> Result<Ability> {
        let remove_items = map_key(self.remove_items, item_names)?;
        let require_items = if self.require_items.is_empty() {
            remove_items.clone()
        } else {
            let req_items = map_key(self.require_items, item_names)?;
            for (k, v) in &remove_items {
                match req_items.get(k) {
                    Some(count) => {
                        if count < v {
                            Err(anyhow!(format!(
                                "cannot remove {:?} of {:?} since we require {:?} of it.",
                                v, k, count
                            )))
                        } else {
                            Ok(())
                        }
                    }
                    None => Err(anyhow!(format!(
                        "cannot remove {:?} because we do not require it.",
                        k
                    ))),
                }?;
            }
            req_items
        };

        Ok(Ability {
            name,
            destroy_item: self.destroy_item,
            stun: self.stun,
            charge: self.charge,
            repeat: self.repeat,
            health: self.health,
            energy: self.energy,
            damage: map_key(self.damage, dmg_types)?,
            block: map_key(self.block, dmg_types)?,
            counter: map_key(self.counter, dmg_types)?,
            require_items,
            remove_items,
            make_items: map_key(self.make_items, item_names)?,
            xp_cost: self.xp_cost,
            accuracy: self.accuracy,
            text: self.text,
            self_text: self.self_text
        })
    }
}

#[derive(Deserialize, Debug)]
pub struct BuffsDeser {
    #[serde(default = "empty_hmap")]
    defense_buffs: HashMap<String, f64>,
    #[serde(default = "empty_hmap")]
    attack_buffs: HashMap<String, f64>,
    #[serde(default = "empty_hmap")]
    stat_buffs: HashMap<String, f64>,
}

impl BuffsDeser {
    pub fn new() -> Self {
        BuffsDeser {
            defense_buffs: HashMap::new(),
            attack_buffs: HashMap::new(),
            stat_buffs: HashMap::new(),
        }
    }

    pub fn into_buffs(
        self,
        dmg_types: &HashSet<DmgType>,
        stat_types: &HashSet<StatType>,
    ) -> Result<Buffs> {
        Ok(Buffs {
            defense_buffs: map_key(self.defense_buffs, dmg_types)?,
            attack_buffs: map_key(self.attack_buffs, dmg_types)?,
            stat_buffs: map_key(self.stat_buffs, stat_types)?,
        })
    }
}

#[derive(Deserialize, Debug)]
pub struct ItemDeser {
    #[serde(default = "false_bool")]
    wearable: bool,
    #[serde(default = "false_bool")]
    equipable: bool,
    #[serde(default = "zero_i64")]
    xp: i64,
    #[serde(default = "BuffsDeser::new")]
    buffs: BuffsDeser,
    #[serde(default = "empty_hmap")]
    abilities: HashMap<String, AbilityDeser>,
    #[serde(default = "no_description")]
    description: String,
    #[serde(default = "empty_vec")]
    tags: Vec<String>,
}

impl ItemDeser {
    pub fn into_item(
        self,
        dmg_types: &HashSet<DmgType>,
        stat_types: &HashSet<StatType>,
        item_names: &HashSet<ItemName>,
        name: ItemName,
    ) -> Result<Item> {
        let mut abilities = HashMap::new();
        for (k, v) in self.abilities {
            abilities.insert(k.clone(), v.into_ability(k, dmg_types, item_names)?);
        }

        Ok(Item {
            name,
            wearable: self.wearable,
            equipable: self.equipable,
            xp: self.xp,
            buffs: self.buffs.into_buffs(dmg_types, stat_types)?,
            abilities,
            description: self.description,
            tags: self.tags,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Ability {
    pub name: String,
    pub destroy_item: bool,
    pub stun: u64,
    pub charge: u64,
    pub repeat: u64,
    pub health: f64,
    pub energy: f64,
    damage: HashMap<DmgType, f64>,
    block: HashMap<DmgType, f64>,
    counter: HashMap<DmgType, f64>,
    pub require_items: HashMap<ItemName, u64>,
    pub remove_items: HashMap<ItemName, u64>,
    pub make_items: HashMap<ItemName, u64>,
    pub xp_cost: i64,
    pub accuracy: f64,
    pub text: String,
    pub self_text: String,
}

impl Ability {
    pub fn damage(&self, g: &GameData) -> HashMap<DmgType, f64> {
        default_empty_fields(&self.damage, 0.0, &g.dmg)
    }

    pub fn block(&self, g: &GameData) -> HashMap<DmgType, f64> {
        default_empty_fields(&self.block, 1.0, &g.dmg)
    }

    pub fn counter(&self, g: &GameData) -> HashMap<DmgType, f64> {
        default_empty_fields(&self.counter, 0.0, &g.dmg)
    }
}

#[derive(Debug, Clone)]
pub struct Buffs {
    defense_buffs: HashMap<DmgType, f64>,
    attack_buffs: HashMap<DmgType, f64>,
    stat_buffs: HashMap<StatType, f64>,
}

impl Buffs {
    pub fn defense_buffs(&self, g: &GameData) -> HashMap<DmgType, f64> {
        default_empty_fields(&self.defense_buffs, 1.0, &g.dmg)
    }

    pub fn attack_buffs(&self, g: &GameData) -> HashMap<DmgType, f64> {
        default_empty_fields(&self.attack_buffs, 1.0, &g.dmg)
    }

    pub fn stat_buffs(&self, g: &GameData) -> HashMap<StatType, f64> {
        default_empty_fields(&self.stat_buffs, 1.0, &g.stat)
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    pub name: ItemName,
    pub wearable: bool,
    pub equipable: bool,
    pub xp: i64,
    pub buffs: Buffs,
    pub abilities: HashMap<String, Ability>,
    pub description: String,
    pub tags: Vec<String>,
}
