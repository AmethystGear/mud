use crate::gamedata::gamedata::{GameData, StatType};
use anyhow::Result;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct Stat {
    base: HashMap<StatType, f64>,
    buffs: HashMap<StatType, f64>,
}

fn default_empty_fields(
    map: &HashMap<StatType, f64>,
    default: f64,
    stat_types: &HashSet<StatType>,
) -> HashMap<StatType, f64> {
    let mut map = map.clone();
    for field in stat_types {
        if map.get(field).is_none() {
            map.insert(field.clone(), default);
        }
    }
    map
}

impl Stat {
    pub fn new(base: HashMap<StatType, f64>, stat_types: &HashSet<StatType>) -> Self {
        Self {
            base: default_empty_fields(&base, 0.0, stat_types),
            buffs: default_empty_fields(&HashMap::new(), 1.0, stat_types),
        }
    }

    pub fn get<S: Into<String>>(&self, s: String, g: &GameData) -> Result<f64> {
        let stat = StatType::checked_from(s, g)?;
        Ok(self.base.get(&stat).expect("bug") * self.buffs.get(&stat).expect("bug"))
    }

    pub fn add_buffs(&mut self, buffs: &HashMap<StatType, f64>, g: &GameData) {
        let buffs = default_empty_fields(buffs, 1.0, &g.stat);
        for field in &g.stat {
            let current = self.buffs.get(field).expect("bug").clone();
            let new_buff = buffs.get(field).expect("bug").clone();
            self.buffs.insert(field.clone(), current * new_buff);
        }
    }

    pub fn remove_buffs(&mut self, buffs: HashMap<StatType, f64>, g: &GameData) {
        let buffs = default_empty_fields(&buffs, 1.0, &g.stat);
        for field in &g.stat {
            let current = self.buffs.get(field).expect("bug").clone();
            let new_buff = buffs.get(field).expect("bug").clone();
            self.buffs.insert(field.clone(), current / new_buff);
        }
    }
}
