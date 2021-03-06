use crate::gamedata::gamedata::ItemName;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    elems: HashMap<ItemName, u64>
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            elems: HashMap::new(),
        }
    }

    pub fn get(&self, item: &ItemName) -> u64 {
        self.elems.get(item).unwrap_or(&0).clone()
    }

    pub fn change(&mut self, item: ItemName, amount: i64) -> Result<()> {
        let num_item = self.get(&item);
        let new_amount = (num_item as i64) + amount;
        if new_amount < 0 {
            return Err(anyhow!(format!(
                "not enough of {:?} in this inventory!",
                item
            )));
        } else {
            self.set(item.clone(), new_amount as u64);
            if new_amount == 0 {
                self.elems.remove(&item);
            }
        }
        Ok(())
    }

    pub fn add(&mut self, item: ItemName, amount: u64) {
        let current = self.get(&item);
        self.set(item, current + amount);
    }

    pub fn set(&mut self, item: ItemName, num: u64) {
        self.elems.insert(item, num);
    }

    pub fn add_inventory(&mut self, other: &Inventory) {
        self.add_items(&other.elems);
    }

    pub fn add_items(&mut self, other: &HashMap<ItemName, u64>) {
        for (item, count) in other {
            self.set(item.clone(), self.get(&item) + count);
        }
    }

    pub fn contains_inventory(&self, other: &Inventory) -> bool {
        self.contains(&other.elems)
    }

    pub fn contains(&self, other: &HashMap<ItemName, u64>) -> bool {
        for (item, count) in other {
            if &self.get(item) < count {
                return false;
            }
        }
        return true;
    }

    pub fn remove_inventory(&mut self, other: &Inventory) -> Result<()> {
        self.remove_items(&other.elems)
    }

    pub fn remove_items(&mut self, other: &HashMap<ItemName, u64>) -> Result<()> {
        for (item, count) in other {
            self.change(item.clone(), -(*count as i64))?;
        }
        Ok(())
    }

    pub fn size(&self) -> u64 {
        let mut size = 0;
        for (_, v) in &self.elems {
            size += v;
        }
        size
    }

    pub fn items(&self) -> std::collections::hash_map::Keys<ItemName, u64> {
        self.elems.keys()
    }

    pub fn clear(&mut self) {
        self.elems = HashMap::new();
    }

    pub fn to_string(&self) -> String {
        let line = |item: &ItemName| format!("{}: {}", item.0, self.get(item));
        if self.size() == 0 {
            return "nothing".into();
        } else if self.size() == 1 {
            let item = self.items().next().expect("at least one item");
            return format!("a {}", item.0);
        }
        let mut s = "\n".into();
        for item in self.items() {
            s = format!("{}{}\n", s, line(item));
        }
        s
    }
}
