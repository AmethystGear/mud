use crate::{deser::entity::Entity, location::Location};
use std::collections::HashMap;

pub struct SpawnedEntities {
    spawned_entities: HashMap<Location, Entity>,
}

impl SpawnedEntities {
    pub fn new() -> Self {
        SpawnedEntities {
            spawned_entities: HashMap::new(),
        }
    }
}
