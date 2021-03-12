use super::{gamedata::{DmgType, ItemName, StatType},
    serde_defaults::*};
use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[derive(Deserialize, Debug)]
pub struct AbilityDeser {
    #[serde(default = "default_false")]
    destroy_item: bool,
    #[serde(default = "default_u64")]
    stun: u64,
    #[serde(default = "default_u64")]
    charge: u64,
    #[serde(default = "default_u64")]
    repeat: u64,
    #[serde(default = "default_f64")]
    health: f64,
    #[serde(default = "default_f64")]
    energy: f64,

    #[serde(default = "default_hmap")]
    damage: HashMap<String, f64>,
    #[serde(default = "default_hmap")]
    block: HashMap<String, f64>,
    #[serde(default = "default_hmap")]
    counter: HashMap<String, f64>,

    #[serde(default = "default_hmap")]
    require_items: HashMap<String, u64>,
    #[serde(default = "default_hmap")]
    remove_items: HashMap<String, u64>,
    #[serde(default = "default_hmap")]
    make_items: HashMap<String, u64>,
}

impl AbilityDeser {
    pub fn into_ability(
        self,
        dmg_types: &HashSet<DmgType>,
        item_names: &HashSet<ItemName>,
    ) -> Result<Ability> {
        let remove_items = map(self.remove_items, item_names)?;
        let require_items = if self.require_items.is_empty() {
            remove_items.clone()
        } else {
            let req_items = map(self.require_items, item_names)?;
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
            destroy_item: self.destroy_item,
            stun: self.stun,
            charge: self.charge,
            repeat: self.repeat,
            health: self.health,
            energy: self.energy,
            damage: map(self.damage, dmg_types)?,
            block: map(self.block, dmg_types)?,
            counter: map(self.counter, dmg_types)?,
            require_items,
            remove_items,
            make_items: map(self.make_items, item_names)?,
        })
    }
}

#[derive(Deserialize, Debug)]
pub struct BuffsDeser {
    #[serde(default = "default_hmap")]
    defense_buffs: HashMap<String, f64>,
    #[serde(default = "default_hmap")]
    attack_buffs: HashMap<String, f64>,
    #[serde(default = "default_hmap")]
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
            defense_buffs: map(self.defense_buffs, dmg_types)?,
            attack_buffs: map(self.attack_buffs, dmg_types)?,
            stat_buffs: map(self.stat_buffs, stat_types)?,
        })
    }
}

#[derive(Deserialize, Debug)]
pub struct ItemDeser {
    #[serde(default = "default_false")]
    wearable: bool,
    #[serde(default = "default_false")]
    equipable: bool,
    #[serde(default = "default_i64")]
    xp: i64,
    #[serde(default = "BuffsDeser::new")]
    buffs: BuffsDeser,
    #[serde(default = "default_hmap")]
    abilities: HashMap<String, AbilityDeser>,
    #[serde(default = "default_string")]
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
            abilities.insert(k, v.into_ability(dmg_types, item_names)?);
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
    destroy_item: bool,
    stun: u64,
    charge: u64,
    repeat: u64,
    health: f64,
    energy: f64,
    damage: HashMap<DmgType, f64>,
    block: HashMap<DmgType, f64>,
    counter: HashMap<DmgType, f64>,
    require_items: HashMap<ItemName, u64>,
    remove_items: HashMap<ItemName, u64>,
    make_items: HashMap<ItemName, u64>,
}

#[derive(Debug, Clone)]
pub struct Buffs {
    defense_buffs: HashMap<DmgType, f64>,
    attack_buffs: HashMap<DmgType, f64>,
    stat_buffs: HashMap<StatType, f64>,
}

#[derive(Debug, Clone)]
pub struct Item {
    name: ItemName,
    wearable: bool,
    equipable: bool,
    xp: i64,
    buffs: Buffs,
    abilities: HashMap<String, Ability>,
    description: Option<String>,
}
