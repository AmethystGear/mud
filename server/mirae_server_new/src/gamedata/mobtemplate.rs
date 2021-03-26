use super::{
    gamedata::{DmgType, GameData, ItemName, MobName, StatType},
    item::{Ability, AbilityDeser},
    serde_defaults::*,
};
use crate::stat::Stat;
use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[derive(Deserialize, Debug)]
struct ItemGenDeser {
    name: String,
    prob: f64,
    per: u64,
}

impl ItemGenDeser {
    pub fn into_itemgen(self, items: &HashSet<ItemName>) -> Result<ItemGen> {
        let name = ItemName::from(self.name);
        if items.contains(&name) {
            Ok(ItemGen {
                name,
                prob: self.prob,
                per: self.per,
            })
        } else {
            Err(anyhow!(format!("invalid item name: {:?}", name)))
        }
    }
}

#[derive(Debug, Clone)]
pub struct ItemGen {
    pub name: ItemName,
    pub prob: f64,
    pub per: u64,
}

#[derive(Deserialize, Debug)]
struct InventoryBuilderDeser {
    min: u64,
    max: u64,
    items: Vec<ItemGenDeser>,
}

#[derive(Debug, Clone)]
pub struct InventoryBuilder {
    pub min: u64,
    pub max: u64,
    pub items: Vec<ItemGen>,
}

impl InventoryBuilderDeser {
    fn new() -> Self {
        InventoryBuilderDeser {
            min: 0,
            max: 0,
            items: vec![],
        }
    }

    fn into_inventorybuilder(self, items: &HashSet<ItemName>) -> Result<InventoryBuilder> {
        let mut new_items = vec![];
        for item in self.items {
            new_items.push((item.into_itemgen(items))?);
        }

        Ok(InventoryBuilder {
            min: self.min,
            max: self.max,
            items: new_items,
        })
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Quotes {
    #[serde(default = "empty_vec")]
    entrance: Vec<String>,
    #[serde(default = "empty_vec")]
    mob_attack: Vec<String>,
    #[serde(default = "empty_vec")]
    player_attack: Vec<String>,
    #[serde(default = "empty_vec")]
    player_run: Vec<String>,
    #[serde(default = "empty_vec")]
    player_run_success: Vec<String>,
    #[serde(default = "empty_vec")]
    player_run_failure: Vec<String>,
    #[serde(default = "empty_vec")]
    player_victory: Vec<String>,
    #[serde(default = "empty_vec")]
    mob_victory: Vec<String>,
}

impl Quotes {
    fn new() -> Self {
        Self {
            entrance: Vec::new(),
            mob_attack: Vec::new(),
            player_attack: Vec::new(),
            player_run: Vec::new(),
            player_run_success: Vec::new(),
            player_run_failure: Vec::new(),
            player_victory: Vec::new(),
            mob_victory: Vec::new(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct MobTemplateDeser {
    #[serde(default = "zero_i64")]
    xp: i64,
    #[serde(default = "empty_hmap")]
    abilities: HashMap<String, AbilityDeser>,
    #[serde(default = "Quotes::new")]
    quotes: Quotes,
    #[serde(default = "InventoryBuilderDeser::new")]
    tools: InventoryBuilderDeser,
    #[serde(default = "InventoryBuilderDeser::new")]
    drops: InventoryBuilderDeser,
    #[serde(default = "empty_hmap")]
    stats: HashMap<String, f64>,
    #[serde(default = "default_jpg_str")]
    display: String,
}

#[derive(Debug, Clone)]
pub struct MobTemplate {
    pub name: MobName,
    pub xp: i64,
    pub abilities: HashMap<String, Ability>,
    pub quotes: Quotes,
    pub tools: InventoryBuilder,
    pub drops: InventoryBuilder,
    pub stats: Stat,
    pub display: String,
}

impl MobTemplateDeser {
    pub fn into_mobtemplate(
        self,
        dmg_types: &HashSet<DmgType>,
        item_names: &HashSet<ItemName>,
        stat_types: &HashSet<StatType>,
        name: MobName,
    ) -> Result<MobTemplate> {
        let mut abilities = HashMap::new();
        for (k, v) in self.abilities {
            abilities.insert(k, v.into_ability(dmg_types, item_names)?);
        }
        let base = map(self.stats, stat_types)?;
        Ok(MobTemplate {
            name,
            xp: self.xp,
            abilities,
            quotes: self.quotes,
            tools: self.tools.into_inventorybuilder(item_names)?,
            drops: self.drops.into_inventorybuilder(item_names)?,
            stats: Stat::new(base, stat_types),
            display: self.display,
        })
    }
}
