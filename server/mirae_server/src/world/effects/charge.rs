#[derive(Debug, Deserialize)]
pub struct Charge {
    num_turns : u64
}

impl Verify for Charge {
    fn default() -> Self {
        Charge {
            num_turns: 0
        }
    }
}

impl Effect for Charge {
    fn do_effect(&self, ability: &Ability, creature: &dyn Creature) {
        todo!()
    }
}