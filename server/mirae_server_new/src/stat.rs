use crate::gamedata::gamedata::{GameData, StatType};
use anyhow::{anyhow, Result};
use std::{hash::Hash, collections::{HashMap, HashSet}};

#[derive(Debug, Clone)]
pub struct Stat {
    health: f64,
    energy: f64,
    base: HashMap<StatType, f64>,
    upgrades: HashMap<StatType, f64>,
}

pub fn default_empty_fields<A : Hash + Eq + Clone, B: Clone>(
    map: &HashMap<A, B>,
    default: B,
    fields: &HashSet<A>,
) -> HashMap<A, B> {
    let mut map = map.clone();
    for field in fields {
        if map.get(field).is_none() {
            map.insert(field.clone(), default.clone());
        }
    }
    map
}

impl Stat {
    pub fn new(
        base: HashMap<StatType, f64>,
        stat_types: &HashSet<StatType>
    ) -> Result<Self> {
        let base = default_empty_fields(&base, 0.0, stat_types);
        let upgrades = default_empty_fields(&HashMap::new(), 0.0, stat_types);
        let max_health = StatType::from("max_health".to_string());
        let max_energy = StatType::from("max_energy".to_string());
        Ok(Self {
            health: base
                .get(&max_health)
                .expect("base stats doesn't have max health?")
                .clone(),
            energy: base
                .get(&max_energy)
                .expect("base stats doesn't have max energy?")
                .clone(),
            base,
            upgrades,
        })
    }

    pub fn get<S: Into<String>>(&self, s: S, g: &GameData) -> Result<f64> {
        let stat = StatType::checked_from(s.into(), g)?;
        Ok(self.base.get(&stat).expect("bug") + self.upgrades.get(&stat).expect("bug"))
    }

    pub fn add_buffs(&mut self, buffs: &HashMap<StatType, f64>, g: &GameData) {
        let buffs = default_empty_fields(buffs, 1.0, &g.stat);
        for field in &g.stat {
            let current = self.base.get(field).expect("bug").clone();
            let new_buff = buffs.get(field).expect("bug").clone();
            self.base.insert(field.clone(), current * new_buff);
        }
        self.bound_health_and_energy(&g);
    }

    pub fn remove_buffs(&mut self, buffs: &HashMap<StatType, f64>, g: &GameData) {
        let buffs = default_empty_fields(buffs, 1.0, &g.stat);
        for field in &g.stat {
            let current = self.base.get(field).expect("bug").clone();
            let new_buff = buffs.get(field).expect("bug").clone();
            self.base.insert(field.clone(), current / new_buff);
        }
        self.bound_health_and_energy(g);
    }

    fn bound_health_and_energy(&mut self, g: &GameData) {
        let max_health = self
            .get("max_health", g)
            .expect("max_health should be a stat");
        let max_energy = self
            .get("max_energy", g)
            .expect("max_energy should be a stat");
        if self.health > max_health {
            self.health = max_health;
        }
        if self.energy > max_energy {
            self.health = max_health;
        }
    }

    pub fn health(&self) -> f64 {
        self.health
    }

    pub fn energy(&self) -> f64 {
        self.energy
    }

    pub fn change_health(&mut self, delta: f64, g: &GameData) {
        let max_health = self
            .get("max_health", g)
            .expect("max_health should be a stat");
        self.health = (self.health + delta).min(max_health);
    }

    pub fn change_energy(&mut self, delta: f64, g: &GameData) -> Result<()> {
        let max_energy = self
            .get("max_energy", g)
            .expect("max_energy should be a stat");
        let new_energy = (self.health + delta).min(max_energy);
        if new_energy < 0.0 {
            return Err(anyhow!("not enough energy!"));
        }
        self.energy = new_energy;
        Ok(())
    }

    pub fn upgrade<S: Into<String>>(&mut self, stat : S, g: &GameData) -> Result<()> {
        let s = stat.into();
        let stat = StatType::checked_from(s.clone(), g)?;
        let val = self.upgrades.get(&stat).expect("bug") + 1.0;
        self.upgrades.insert(stat, val);
        Ok(())
    }
}
