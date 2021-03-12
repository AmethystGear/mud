use anyhow::{anyhow, Result};
use std::collections::HashMap;

pub struct Inventory(pub HashMap<String, u64>);

impl Inventory {
    pub fn get(&self, item: &String) -> u64 {
        if let Some(num) = self.0.get(item) {
            num.clone()
        } else {
            0
        }
    }

    pub fn change(&mut self, item: String, amount: i64) -> Result<()> {
        let num_item = self.get(&item);
        if ((num_item as i64) + amount) < 0 {
            return Err(anyhow!("not enough of that item in this inventory!"));
        } else if ((num_item as i64) + amount) == 0 {
            self.0.remove(&item);
        } else {
            self.set(item, ((num_item as i64) + amount) as u64);
        }
        Ok(())
    }

    pub fn set(&mut self, item: String, num: u64) {
        self.0.insert(item, num);
    }
}
