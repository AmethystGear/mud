#[derive(Debug, Deserialize)]
pub struct Repeat {
    num_turns : u64
}

impl Verify for Repeat {
    fn default() -> Self {
        Repeat {
            num_turns: 0
        }
    }
}

impl Effect for Repeat {
    fn do_effect(&self, ability: &Ability, creature: &dyn Creature) {
        todo!()
    }
}
