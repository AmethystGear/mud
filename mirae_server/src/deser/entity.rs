use super::stats::Stats;
use crate::location::Vector2;
use crate::{gamedata, inventory, trades::Trades};
use anyhow::{anyhow, Result};
use inventory::Inventory;
use rand::{prelude::StdRng, Rng, SeedableRng};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct Quotes {
    pub entrance: Vec<String>,
    pub attack: Vec<String>,
    pub player_run: Vec<String>,
    pub player_victory: Vec<String>,
    pub mob_victory: Vec<String>,
}

impl Quotes {
    fn new() -> Self {
        Quotes {
            entrance: vec![],
            attack: vec![],
            player_run: vec![],
            player_victory: vec![],
            mob_victory: vec![],
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RandItem {
    items: Vec<String>,
    prob: Vec<f64>,
    per: Vec<u64>,
    range: Vec<u64>,
}

impl RandItem {
    fn new() -> Self {
        RandItem {
            items: Vec::new(),
            prob: Vec::new(),
            per: Vec::new(),
            range: Vec::new(),
        }
    }

    fn to_inventory(&self, rng: &mut StdRng) -> Result<Inventory> {
        let mut num_items = rng.gen_range(
            self.range
                .get(0)
                .ok_or_else(|| anyhow!("index out of bounds"))?,
            self.range
                .get(1)
                .ok_or_else(|| anyhow!("index out of bounds"))?
                + 1,
        );
        let mut items: HashMap<String, u64> = HashMap::new();
        while num_items > 0 {
            let item = rng.gen_range(0, self.items.len());
            let prob = self.prob[item];
            if rng.gen::<f64>() < prob {
                let per = self.per[item];
                let item = &self.items[item];
                if let Some(val) = items.get(item) {
                    let val = val.clone();
                    items.insert(item.to_string(), per + val);
                } else {
                    items.insert(item.to_string(), per);
                }
                num_items -= 1;
            }
        }
        Ok(Inventory(items))
    }
}

#[derive(Debug, Deserialize)]
pub struct RandTrade {
    items: HashMap<String, String>,
    range: Vec<u64>,
}

impl RandTrade {
    fn new() -> Self {
        RandTrade {
            items: HashMap::new(),
            range: Vec::new(),
        }
    }

    fn to_trades(&self, rng: &mut StdRng) -> Result<Trades> {
        let num_trades = rng.gen_range(
            self.range
                .get(0)
                .ok_or_else(|| anyhow!("index out of bounds"))?,
            self.range
                .get(1)
                .ok_or_else(|| anyhow!("index out of bounds"))?
                + 1,
        );
        let mut trades = HashMap::new();
        let set: Vec<String> = self.items.keys().cloned().collect();
        for _ in 0..num_trades {
            let item = rng.gen_range(0, set.len());
            let in_trade = set[item].clone();
            let out_trade = self
                .items
                .get(&set[item])
                .ok_or_else(|| {
                    anyhow!("this should never happen, key should always be in hashmap")
                })?
                .clone();
            trades.insert(in_trade, out_trade);
        }
        Ok(Trades(trades))
    }
}

fn default_display() -> String {
    "default.png".to_string()
}

fn default_i64() -> i64 {
    0
}

#[derive(Debug, Deserialize, Clone)]
pub struct Steal {
    pub max_num_items: u64,
    pub looting: f64,
    pub intelligence: f64,
}

impl Steal {
    pub fn new() -> Self {
        Self {
            max_num_items: 0,
            looting: 0.0,
            intelligence: 0.0,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SpecialBoost {
    pub chance: f64,
    pub boost: f64,
}

impl SpecialBoost {
    pub fn new() -> Self {
        Self {
            chance: -1.0,
            boost: 1.0,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Regen {
    pub health_regen: i64,
    pub energy_regen: i64,
}

#[derive(Debug, Deserialize)]
pub struct EntityTemplate {
    #[serde(default = "Quotes::new")]
    pub quotes: Quotes,
    #[serde(default = "default_display")]
    pub display: String,
    #[serde(default = "RandItem::new")]
    pub drops: RandItem,
    #[serde(default = "RandItem::new")]
    pub tools: RandItem,
    #[serde(default = "RandTrade::new")]
    pub trades: RandTrade,
    #[serde(default = "default_i64")]
    pub xp: i64,
    #[serde(default = "Steal::new")]
    pub steal: Steal,
    #[serde(default = "SpecialBoost::new")]
    pub boost: SpecialBoost,
    pub stat: Stats,
}

impl EntityTemplate {
    pub fn construct(&self, location: Vector2, name: String, seed: u64) -> Result<Entity> {
        let mut rng = StdRng::seed_from_u64(seed);

        let mut boosted_stat = None;
        let stats = ["intelligence", "looting", "speed", "energy", "health"];
        if self.boost.chance > rng.gen::<f64>() {
            let stat = stats[rng.gen_range(0, stats.len())];
            boosted_stat = Some(stat);
        }

        let mut steal = self.steal.clone();
        let mut stat = self.stat.clone();
        match boosted_stat {
            Some(string) => {
                match string {
                    "intelligence" => steal.intelligence *= self.boost.boost,
                    "looting" => steal.looting *= self.boost.boost,
                    "speed" => stat.speed *= self.boost.boost as u64,
                    "energy" => stat.energy *= self.boost.boost as u64,
                    "health" => stat.health *= self.boost.boost as u64,
                    _ => {}
                };
            }
            _ => {}
        };
        let boosted_stat = boosted_stat.map(|x| x.to_string());
        Ok(Entity {
            location,
            name,
            quotes: self.quotes.clone(),
            xp: self.xp,
            tools_inventory: self.tools.to_inventory(&mut rng)?,
            drops_inventory: self.drops.to_inventory(&mut rng)?,
            trades: self.trades.to_trades(&mut rng)?,
            steal,
            stat,
            boosted_stat,
        })
    }
}

pub struct Entity {
    pub location: Vector2,
    pub name: String,
    pub quotes: Quotes,
    pub xp: i64,
    pub tools_inventory: Inventory,
    pub drops_inventory: Inventory,
    pub trades: Trades,
    pub steal: Steal,
    pub boosted_stat: Option<String>,
    pub stat: Stats,
}
