use crate::{gamedata::mobtemplate::MobTemplate, vector3::Vector3};

#[derive(Debug, Clone)]
pub struct Mob {
    id : u64
}

impl Mob {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn new(id : u64, loc : Vector3, template : &MobTemplate) -> Self {
        Self {
            id
        }
    }
}
