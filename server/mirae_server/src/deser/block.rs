use serde::Deserialize;

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
