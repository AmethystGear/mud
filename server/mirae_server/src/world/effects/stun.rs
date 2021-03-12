#[derive(Debug, Deserialize)]
pub struct Stun {
    num_turns : u64
}

impl Verify for Stun {
    fn default() -> Self {
        Stun {
            num_turns: 0
        }
    }
}