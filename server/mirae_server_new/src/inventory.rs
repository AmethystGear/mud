use anyhow::{anyhow, Result};
use std::collections::HashMap;
use crate::gamedata::gamedata::ItemName;

pub struct Inventory(pub HashMap<ItemName, u64>);

impl Inventory {
    pub fn get(&self, item: &ItemName) -> u64 {
        if let Some(num) = self.0.get(item) {
            num.clone()
        } else {
            0
        }
    }

    pub fn change(&mut self, item: ItemName, amount: i64) -> Result<()> {
        let num_item = self.get(&item);
        if ((num_item as i64) + amount) < 0 {
            return Err(anyhow!(format!(
                "not enough of {:?} in this inventory!",
                item
            )));
        } else if ((num_item as i64) + amount) == 0 {
            self.0.remove(&item);
        } else {
            self.set(item, ((num_item as i64) + amount) as u64);
        }
        Ok(())
    }

    pub fn set(&mut self, item: ItemName, num: u64) {
        self.0.insert(item, num);
    }
}
