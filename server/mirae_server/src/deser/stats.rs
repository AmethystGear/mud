use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Stats {
    pub health: u64,
    pub energy: u64,
    pub speed: u64,
}
