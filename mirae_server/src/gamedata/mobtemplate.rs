use super::{
    gamedata::{DmgType, ItemName, MobName, StatType},
    item::{Ability, AbilityDeser},
    serde_defaults::*,
};
use crate::stat::Stat;
use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct Trade {
    pub in_item: ItemName,
    pub out_item: ItemName,
    pub in_cnt: u64,
    pub out_cnt: u64,
}

#[derive(Debug, Deserialize)]
struct TradeDeser {
    in_item: String,
    out_item: String,
    in_cnt: u64,
    out_cnt: u64,
}

impl TradeDeser {
    pub fn into_trade(self, items: &HashSet<ItemName>) -> Result<Trade> {
        let in_item = ItemName::from(self.in_item);
        let out_item = ItemName::from(self.out_item);
        if items.contains(&in_item) && items.contains(&out_item) {
            Ok(Trade {
                in_item,
                out_item,
                in_cnt: self.in_cnt,
                out_cnt: self.out_cnt,
            })
        } else {
            Err(anyhow!(format!(
                "possibly invalid item name(s): {:?}, {:?}",
                in_item, out_item
            )))
        }
    }
}

#[derive(Deserialize, Debug)]
struct ItemGenDeser {
    name: String,
    per: u64,
}

impl ItemGenDeser {
    pub fn into_itemgen(self, items: &HashSet<ItemName>) -> Result<ItemGen> {
        let name = ItemName::from(self.name);
        if items.contains(&name) {
            Ok(ItemGen {
                name,
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
    pub per: u64,
}

#[derive(Deserialize, Debug)]
struct InventoryBuilderDeser {
    #[serde(default = "zero_u64")]
    min: u64,
    #[serde(default = "zero_u64")]
    max: u64,
    #[serde(default = "empty_vec")]
    tags: Vec<String>,
    #[serde(default = "empty_vec")]
    required_items: Vec<ItemGenDeser>,
    #[serde(default = "empty_vec")]
    items: Vec<Vec<ItemGenDeser>>,
    #[serde(default = "empty_vec")]
    probs: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct InventoryBuilder {
    pub min: u64,
    pub max: u64,
    pub tags: Vec<String>,
    pub required_items: Vec<ItemGen>,
    pub items: Vec<Vec<ItemGen>>,
    pub probs: Vec<f64>,
}

impl InventoryBuilderDeser {
    fn new() -> Self {
        InventoryBuilderDeser {
            min: 0,
            max: 0,
            tags: vec![],
            required_items: vec![],
            items: vec![],
            probs: vec![],
        }
    }

    fn into_inventorybuilder(self, items: &HashSet<ItemName>) -> Result<InventoryBuilder> {
        let text = format!("{:#?}", self);
        let mut n_items = vec![];
        for item_list in self.items {
            let mut new_items = vec![];
            for item in item_list {
                new_items.push((item.into_itemgen(items))?);
            }
            n_items.push(new_items);
        }

        let mut req_items = vec![];
        for item in self.required_items {
            req_items.push(item.into_itemgen(items)?);
        }

        if n_items.len() != self.probs.len() {
            return Err(anyhow!(format!("probs len != items len\n{}", text)));
        }

        Ok(InventoryBuilder {
            min: self.min,
            max: self.max,
            items: n_items,
            probs: self.probs,
            required_items: req_items,
            tags: self.tags,
        })
    }
}

fn vec_with_default_quote() -> Vec<String> {
    vec!["...".into()]
}

#[derive(Deserialize, Debug, Clone)]
pub struct Quotes {
    #[serde(default = "vec_with_default_quote")]
    pub entrance: Vec<String>,
    #[serde(default = "vec_with_default_quote")]
    pub attack: Vec<String>,
    #[serde(default = "vec_with_default_quote")]
    pub run: Vec<String>,
    #[serde(default = "vec_with_default_quote")]
    pub player_victory: Vec<String>,
    #[serde(default = "vec_with_default_quote")]
    pub mob_victory: Vec<String>,
}

impl Quotes {
    fn new() -> Self {
        Self {
            entrance: vec_with_default_quote(),
            attack: vec_with_default_quote(),
            run: vec_with_default_quote(),
            player_victory: vec_with_default_quote(),
            mob_victory: vec_with_default_quote(),
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
    #[serde(default = "default_png_string")]
    display: String,
    #[serde(default = "default_png_string")]
    display_img: String,
    #[serde(default = "empty_string")]
    scan: String,
    #[serde(default = "no_description")]
    descr: String,
    #[serde(default = "empty_vec")]
    trades: Vec<TradeDeser>,
    #[serde(default = "empty_vec")]
    tags: Vec<String>,
    #[serde(default = "false_bool")]
    dont_spawn: bool,
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
    pub display_img: String,
    pub scan: String,
    pub description: String,
    pub trades: Vec<Trade>,
    pub tags: Vec<String>,
    pub dont_spawn: bool,
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
            abilities.insert(k.clone(), v.into_ability(k, dmg_types, item_names)?);
        }
        let base = map_key(self.stats, stat_types)?;

        let mut trades = Vec::new();
        for t in self.trades {
            trades.push(t.into_trade(item_names)?);
        }

        Ok(MobTemplate {
            name,
            xp: self.xp,
            abilities,
            quotes: self.quotes,
            tools: self.tools.into_inventorybuilder(item_names)?,
            drops: self.drops.into_inventorybuilder(item_names)?,
            stats: Stat::new(base, stat_types)?,
            display: self.display,
            display_img: self.display_img,
            scan: self.scan,
            description: self.descr,
            trades,
            tags: self.tags,
            dont_spawn: self.dont_spawn,
        })
    }
}
