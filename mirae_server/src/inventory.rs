use crate::gamedata::gamedata::ItemName;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    elems: HashMap<ItemName, u64>,
    size: u64,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            elems: HashMap::new(),
            size: 0,
        }
    }

    pub fn get(&self, item: &ItemName) -> u64 {
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
            self.set(item, ((num_item as i64) + amount) as u64);
        }
        self.size = (self.size as i64 + amount) as u64;
        Ok(())
    }

    pub fn add(&mut self, item: ItemName, amount: u64) {
        let current = self.get(&item);
        self.set(item, current + amount);
        self.size += amount;
    }

    pub fn set(&mut self, item: ItemName, num: u64) {
        let current = self.get(&item);
        self.elems.insert(item, num);
        self.size -= current;
        self.size += num;
    }

    pub fn add_inventory(&mut self, other: &Inventory) {
        self.add_items(&other.elems);
    }

    pub fn add_items(&mut self, other: &HashMap<ItemName, u64>) {
        for (item, count) in other {
            self.size += count;
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
        self.size
    }

    pub fn items(&self) -> std::collections::hash_map::Keys<ItemName, u64> {
        self.elems.keys()
    }

    pub fn clear(&mut self) {
        self.elems = HashMap::new();
        self.size = 0;
    }

    pub fn to_string(&self) -> String {
        let line = |item: &ItemName| format!("{}: {}", item.0, self.get(item));
        if self.size() == 0 {
            return "nothing".into();
        } else if self.items().len() == 1 {
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
