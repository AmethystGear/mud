use crate::gamedata;
use crate::{
    deser::entity::Entity,
    location::Vector2,
};
use anyhow::{anyhow, Result};
use std::collections::HashMap;

pub struct SpawnedEntities(HashMap<Vector2, Entity>);

impl SpawnedEntities {
    pub fn new() -> Self {
        SpawnedEntities(HashMap::new())
    }

    pub fn mov(&mut self, start: Vector2, end: Vector2) -> Result<()> {
        if let Some(_) = self.0.get(&end) {
            Err(anyhow!(
                "cannot move from {:?} to {:?} because there is already an entity at {:?}",
                start,
                end,
                end
            ))
        } else {
            if let Some(entity) = self.0.remove(&start) {
                self.0.insert(end, entity);
                Ok(())
            } else {
                Err(anyhow!(
                    "cannot move from {:?} to {:?} because there is no entity at {:?}",
                    start,
                    end,
                    start
                ))
            }
        }
    }

    pub fn del(&mut self, loc: Vector2) -> Result<()> {
        if let Some(_) = self.0.get(&loc) {
            self.0.remove(&loc);
            Ok(())
        } else {
            Err(anyhow!(
                "cannot delete entity at {:?} because there is no entity at {:?}",
                loc,
                loc
            ))
        }
    }

    pub fn spawn(&mut self, entity_name: String, loc: Vector2, seed: u64) -> Result<()> {
        if let Some(_) = self.0.get(&loc) {
            Err(anyhow!(
                "cannot create entity at {:?} because there already is an entity at {:?}",
                loc,
                loc
            ))
        } else {
            let template = gamedata::GAMEDATA
                .entities
                .get(&entity_name)
                .ok_or_else(|| anyhow!("invalid entity name!"))?;
            let entity = template.construct(loc, entity_name, seed)?;
            self.0.insert(loc, entity);
            Ok(())
        }
    }

    pub fn get(&mut self, loc : Vector2) -> Result<&mut Entity> {
        self.0.get_mut(&loc).ok_or_else(|| anyhow!("no entity at the location {:?}", loc))
    }
}
