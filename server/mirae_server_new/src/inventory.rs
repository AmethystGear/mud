use crate::gamedata::gamedata::ItemName;
use anyhow::{anyhow, Result};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Inventory {
    elems: HashMap<ItemName, usize>,
    size: usize,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            elems: HashMap::new(),
            size: 0,
        }
    }

    pub fn get(&self, item: &ItemName) -> usize {
        self.elems.get(item).unwrap_or(&0).clone()
    }

    pub fn change(&mut self, item: ItemName, amount: i64) -> Result<()> {
        let num_item = self.get(&item);
        if ((num_item as i64) + amount) < 0 {
            return Err(anyhow!(format!(
                "not enough of {:?} in this inventory!",
                item
            )));
        } else if ((num_item as i64) + amount) == 0 {
            self.elems.remove(&item);
        } else {
            self.set(item, ((num_item as i64) + amount) as usize);
        }
        self.size = (self.size as i64 + amount) as usize;
        Ok(())
    }

    pub fn add(&mut self, item: ItemName, amount: usize) {
        let current = self.get(&item);
        self.set(item, current + amount);
    }

    pub fn set(&mut self, item: ItemName, num: usize) {
        self.elems.insert(item, num);
    }

    pub fn add_inventory(&mut self, other: &Inventory) {
        for (item, count) in &other.elems {
            self.size += count;
            self.set(item.clone(), self.get(&item) + count);
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn items(&self) -> std::collections::hash_map::Keys<ItemName, usize> {
        self.elems.keys()
    }
}
