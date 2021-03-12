use super::{
    gamedata::{DmgType, ItemName, MobAction, MobName},
    item::{Ability, AbilityDeser},
    serde_defaults::*,
};
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
struct ItemGen {
    name: ItemName,
    prob: f64,
    per: u64,
}

#[derive(Deserialize, Debug)]
struct InventoryBuilderDeser {
    min: u64,
    max: u64,
    items: Vec<ItemGenDeser>,
}

#[derive(Debug, Clone)]
struct InventoryBuilder {
    min: u64,
    max: u64,
    items: Vec<ItemGen>,
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

#[derive(Deserialize, Debug)]
pub struct MobTemplateDeser {
    #[serde(default = "default_i64")]
    xp: i64,
    #[serde(default = "default_hmap")]
    abilities: HashMap<String, AbilityDeser>,
    #[serde(default = "default_hmap")]
    quotes: HashMap<String, Vec<String>>,
    #[serde(default = "InventoryBuilderDeser::new")]
    tools: InventoryBuilderDeser,
    #[serde(default = "InventoryBuilderDeser::new")]
    drops: InventoryBuilderDeser,
}

#[derive(Debug, Clone)]
pub struct MobTemplate {
    name: MobName,
    xp: i64,
    abilities: HashMap<String, Ability>,
    quotes: HashMap<MobAction, Vec<String>>,
    tools: InventoryBuilder,
    drops: InventoryBuilder,
}

impl MobTemplateDeser {
    pub fn into_mobtemplate(
        self,
        dmg_types: &HashSet<DmgType>,
        item_names: &HashSet<ItemName>,
        mob_actions: &HashSet<MobAction>,
        name: MobName,
    ) -> Result<MobTemplate> {
        let mut abilities = HashMap::new();
        for (k, v) in self.abilities {
            abilities.insert(k, v.into_ability(dmg_types, item_names)?);
        }
        Ok(MobTemplate {
            name,
            xp: self.xp,
            abilities,
            quotes: map(self.quotes, mob_actions)?,
            tools: self.tools.into_inventorybuilder(item_names)?,
            drops: self.drops.into_inventorybuilder(item_names)?,
        })
    }
}