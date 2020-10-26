use crate::gamedata;
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

lazy_static! {
    static ref NAMESETS: HashMap<NameSet, HashSet<String>> = {
        let mut m: HashMap<NameSet, HashSet<String>> = HashMap::new();
        m.insert(NameSet::Block, gamedata::GAMEDATA.blocks.keys().cloned().collect());
        m.insert(NameSet::Entity, gamedata::GAMEDATA.entities.keys().cloned().collect());
        m.insert(NameSet::Item, gamedata::GAMEDATA.items.keys().cloned().collect());
        m.insert(NameSet::Dmg, gamedata::GAMEDATA.dmg.iter().cloned().collect());
        m.insert(NameSet::Effect, gamedata::GAMEDATA.effect.keys().cloned().collect());
        m.insert(NameSet::Stat, gamedata::GAMEDATA.stat.iter().cloned().collect());
        return m;
    };
}

#[derive(Hash, Eq, PartialEq)]
pub enum NameSet {
    Block,
    Entity,
    Item,
    Effect,
    Dmg,
    Stat,
}

pub trait Named: Sized + Hash + Clone + Debug {
    fn name(&self) -> String;
    fn __from_name(s: String) -> Self;
    fn __name_set() -> NameSet;

    fn new<S: Into<String>>(name: S) -> Result<Self> {
        let name = name.into();
        let valid_names = NAMESETS.get(&Self::__name_set()).unwrap();
        if valid_names.contains(&name) {
            Ok(Self::__from_name(name))
        } else {
            Err(anyhow!(
                "name {} not in valid_names {:?}",
                name,
                valid_names
            ))
        }
    }
}
