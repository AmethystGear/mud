
#[derive(Debug, Deserialize)]
pub struct Regen {
    health : i64,
    energy : i64
}

impl Verify for Regen {
    fn default() -> Self {
        Regen {
            health : 0,
            energy : 0
        }
    }
}
