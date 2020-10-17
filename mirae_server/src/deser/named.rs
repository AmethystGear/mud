use crate::gamedata;
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};

lazy_static! {
    static ref NAMESETS: HashMap<NameSet, HashSet<String>> = {
        let mut m: HashMap<NameSet, HashSet<String>> = HashMap::new();
        let gamedata = gamedata::get().unwrap();
        if let Ok(gamedata) = &*gamedata {
            m.insert(NameSet::Block, gamedata.blocks.keys().cloned().collect());
            m.insert(NameSet::Entity, gamedata.entities.keys().cloned().collect());
            m.insert(NameSet::Item, gamedata.items.keys().cloned().collect());
        } else {
            panic!("can't access gamedata!")
        }
        return m;
    };
}

#[derive(Hash, Eq, PartialEq)]
pub enum NameSet {
    Block,
    Entity,
    Item,
}

pub trait Named: Sized {
    fn name(&self) -> String;

    fn from_name(s: String) -> Self;

    fn new<S: Into<String>>(name: S, name_set: NameSet) -> Result<Self> {
        let name = name.into();
        let valid_names = NAMESETS.get(&name_set).unwrap();
        if valid_names.contains(&name) {
            Ok(Self::from_name(name))
        } else {
            Err(anyhow!(
                "name {} not in valid_names {:?}",
                name,
                valid_names
            ))
        }
    }
}
