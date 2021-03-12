use serde::Deserialize;
use crate::gamedata;
use anyhow::{Result, anyhow};
use std::collections::HashSet;
use serde_jacl::Value;

pub struct Effect {

}

#[derive(Debug, Deserialize)]
pub struct AbilityDeser {
    energy_cost: u64,
    #[serde(default = "Stun::default")]
    stun: Stun,
    #[serde(default = "Charge::default")]
    charge: Charge,
    #[serde(default = "Repeat::default")]
    repeat: Repeat,
    #[serde(default = "Damage::default")]
    damage: Damage,
    #[serde(default = "Block::default")]
    block: Block,
    #[serde(default = "Counter::default")]
    counter: Counter,
    #[serde(default = "Regen::default")]
    regen: Regen,
    #[serde(default = "DestroyItem::default")]
    destroy_item : DestroyItem
}

impl AbilityDeser {
    pub fn verify(self) -> Result<Ability> {
        if self.stun.verify() && 
           self.charge.verify() &&
           self.repeat.verify() &&
           self.damage.verify() &&
           self.block.verify() &&
           self.counter.verify() &&
           self.regen.verify() &&
           self.destroy_item.verify() {
            let eval: Vec<Box<dyn Effect>> = vec![Box::new(self.charge), Box::new(self.repeat)];
            Ok(Ability(eval))
        } else {
            Err(anyhow!("ability verification failed"))
        }
    }
}

struct Ability(Vec<Box<dyn Effect>>);
