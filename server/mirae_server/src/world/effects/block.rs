use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Block {
    proportions : HashMap<String, f64>
}

impl Verify for Block {
    fn verify(&self) -> bool {
        verify_dmg_names(self.proportions.keys().cloned().collect())
    }
    fn default() -> Self {
        Block {
            proportions: HashMap::new()
        }
    }
}
