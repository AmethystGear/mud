use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Stats {
    pub health : u64,
    pub energy : u64,
    pub speed : u64
}