use crate::location::Location;
use crate::{inventory, trades::Trades};
use anyhow::{anyhow, Result};
use inventory::Inventory;
use serde::Deserialize;
use serde_jacl::structs::Number;
use std::collections::HashMap;

pub struct Entities(HashMap<String, EntityTemplate>);

pub struct EntityName {
    name: String,
}
impl EntityName {}

#[derive(Debug, Deserialize)]
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
    per: Vec<i64>,
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
}

#[derive(Debug, Deserialize)]
pub struct RandTrade {
    items: HashMap<String, String>,
    min: u64,
    max: u64,
}

impl RandTrade {
    fn new() -> Self {
        RandTrade {
            items: HashMap::new(),
            min: 0,
            max: 0,
        }
    }
}

fn default_display() -> String {
    "default.png".to_string()
}

fn default_i64() -> i64 {
    0
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
    pub items: RandItem,
    #[serde(default = "RandTrade::new")]
    pub trades: RandTrade,
    #[serde(default = "default_i64")]
    pub xp: i64,
    pub stats: HashMap<String, Number>,
}

impl EntityTemplate {
    pub fn construct(self, location: Location, name: EntityName) -> Entity {
        Entity {
            location,
            name,
            quotes: self.quotes,
            xp: self.xp,
            stats: self.stats,
            inventory: Inventory(HashMap::new()),
            trades: Trades(HashMap::new()),
        }
    }
}

pub struct Entity {
    pub location: Location,
    pub name: EntityName,
    pub quotes: Quotes,
    pub xp: i64,
    pub stats: HashMap<String, Number>,
    pub inventory: Inventory,
    pub trades: Trades,
}
