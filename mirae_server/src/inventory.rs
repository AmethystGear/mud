use crate::deser::item::ItemName;
use anyhow::Result;
use std::collections::HashMap;

pub struct Inventory(pub HashMap<ItemName, u64>);

impl Inventory {
    pub fn get(&self, item: ItemName) -> u64 {
        0
    }

    pub fn change(&mut self, item: ItemName, amount: i64) -> Result<()> {
        Ok(())
    }

    pub fn set(&mut self, item: ItemName, num: u64) {}
}
