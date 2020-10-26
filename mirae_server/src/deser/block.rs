use super::named::{NameSet, Named};
use serde::Deserialize;

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct BlockName {
    name: String,
}

impl Named for BlockName {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn __from_name(s: String) -> Self {
        BlockName { name: s }
    }

    fn __name_set() -> NameSet {
        NameSet::Block
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Block {
    pub display: String,
    #[serde(default = "default_mob_spawn_chance")]
    pub mob_spawn_chance: f64,
    #[serde(default = "default_solid")]
    pub solid: bool,
    #[serde(default = "default_mob_filter")]
    pub mob_filter: Vec<String>,
}

fn default_mob_spawn_chance() -> f64 {
    0.0
}

fn default_solid() -> bool {
    false
}

fn default_mob_filter() -> Vec<String> {
    vec![]
}
