#[derive(Debug, Deserialize)]
pub struct Counter {
    proportions : HashMap<String, f64>
}

impl Verify for Counter {
    fn verify(&self) -> bool {
        verify_dmg_names(self.proportions.keys().cloned().collect())
    }
    fn default() -> Self {
        Counter {
            proportions: HashMap::new()
        }
    }
}