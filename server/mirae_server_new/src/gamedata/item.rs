use super::{
    gamedata::{DmgType, ItemName, StatType},
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
            damage: default_empty_fields(&map_key(self.damage, dmg_types)?, 0.0, dmg_types),
            block: default_empty_fields(&map_key(self.block, dmg_types)?, 1.0, dmg_types),
            counter: default_empty_fields(&map_key(self.counter, dmg_types)?, 1.0, dmg_types),
            require_items,
            remove_items,
            make_items: map_key(self.make_items, item_names)?,
            xp_cost: self.xp_cost,
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
            defense_buffs: default_empty_fields(
                &map_key(self.defense_buffs, dmg_types)?,
                1.0,
                dmg_types,
            ),
            attack_buffs: default_empty_fields(
                &map_key(self.attack_buffs, dmg_types)?,
                1.0,
                dmg_types,
            ),
            stat_buffs: default_empty_fields(
                &map_key(self.stat_buffs, stat_types)?,
                1.0,
                stat_types,
            ),
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
    #[serde(default = "empty_string")]
    description: String,
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
            description: if self.description == "" {
                None
            } else {
                Some(self.description)
            },
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
    pub damage: HashMap<DmgType, f64>,
    pub block: HashMap<DmgType, f64>,
    pub counter: HashMap<DmgType, f64>,
    pub require_items: HashMap<ItemName, u64>,
    pub remove_items: HashMap<ItemName, u64>,
    pub make_items: HashMap<ItemName, u64>,
    pub xp_cost: i64,
}

#[derive(Debug, Clone)]
pub struct Buffs {
    pub defense_buffs: HashMap<DmgType, f64>,
    pub attack_buffs: HashMap<DmgType, f64>,
    pub stat_buffs: HashMap<StatType, f64>,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub name: ItemName,
    pub wearable: bool,
    pub equipable: bool,
    pub xp: i64,
    pub buffs: Buffs,
    pub abilities: HashMap<String, Ability>,
    pub description: Option<String>,
}
