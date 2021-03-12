#[derive(Debug, Deserialize)]
pub struct Damage {
    amounts : HashMap<String, f64>
}

impl Verify for Damage {
    fn verify(&self) -> bool {
        verify_dmg_names(self.amounts.keys().cloned().collect())
    }
    fn default() -> Self {
        Damage {
            amounts: HashMap::new()
        }
    }
}