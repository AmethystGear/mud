use super::{entity::Entity, spawned::Spawned, player::Player};
use crate::world::perlin_noise::generate_perlin_noise;
use crate::{
    gamedata,
    location::Vector2, requests::worldupdate::{WorldEntityUpdate, WorldUpdate}
};
use anyhow::{anyhow, Result};
use rand::{prelude::StdRng, Rng, SeedableRng};
use std::collections::{HashMap, HashSet};

mod effects;

#[derive(Debug)]
pub struct Map {
    id_to_name: HashMap<u16, String>,
    name_to_id: HashMap<String, u16>,
    map_size: u16,
    map: Vec<u16>,
    max: u16,
}

impl Map {
    fn new(name_to_item: HashSet<&String>, map_size: u16) -> Result<Self> {
        let mut id_to_name = HashMap::new();
        let mut name_to_id = HashMap::new();
        let mut id: u16 = 0;
        for k in name_to_item {
            name_to_id.insert(k.to_string(), id);
            id_to_name.insert(id, k.to_string());
            id += 1;
        }
        Ok(Map {
            id_to_name,
            name_to_id,
            map_size,
            map: vec![u16::MAX; (map_size as usize) * (map_size as usize)],
            max: id,
        })
    }

    fn index(&self, loc: Vector2) -> Result<usize> {
        if loc.x >= self.map_size || loc.y >= self.map_size {
            Err(anyhow!(
                "point {:?} not in map of size {}",
                loc,
                self.map_size
            ))
        } else {
            Ok((loc.y as usize) * (self.map_size as usize) + (loc.x as usize))
        }
    }

    fn set(&mut self, loc: Vector2, name: Option<&String>) -> Result<()> {
        let index = self.index(loc)?;
        self.direct_set(index, name)
    }

    fn direct_set(&mut self, index: usize, name: Option<&String>) -> Result<()> {
        if let Some(name) = name {
            if let Some(id) = self.name_to_id.get(name) {
                self.map[index] = id.clone();
                Ok(())
            } else {
                Err(anyhow!(
                    "object with name {:?} does not exist in map {:?}",
                    name,
                    self.name_to_id
                ))
            }
        } else {
            self.map[index] = u16::MAX;
            Ok(())
        }
    }

    pub fn get(&self, loc: Vector2) -> Result<Option<&String>> {
        let index = self.index(loc)?;
        if self.map[index] == u16::MAX {
            Ok(None)
        } else {
            if let Some(name) = self.id_to_name.get(&self.map[index]) {
                Ok(Some(name))
            } else {
                Err(anyhow!(
                    "the id {} at {:?}, doesn't have a mapping in {:?}",
                    self.map[index],
                    loc,
                    self.id_to_name
                ))
            }
        }
    }

    pub fn get_id(&self, loc: Vector2) -> Result<Option<u16>> {
        let index = self.index(loc)?;
        if self.map[index] == u16::MAX {
            Ok(None)
        } else {
            Ok(Some(self.map[index]))
        }
    }

    pub fn max(&self) -> u16 {
        self.max
    }

    pub fn name_to_id(&self) -> &HashMap<String, u16> {
        &self.name_to_id
    }

    pub fn id_to_name(&self) -> &HashMap<u16, String> {
        &self.id_to_name
    }
}

pub struct World {
    pub blocks: Map,
    pub entities: Map,
    pub spawned_entities: Spawned<Entity>,
    pub spawned_players: Spawned<Player>,
    rng: StdRng,
}

impl World {
    pub fn from_seed(seed: u64) -> Result<Self> {
        let map_size = gamedata::GAMEDATA.terrain.map_size;
        let rng = StdRng::seed_from_u64(seed);
        let mut world = World {
            blocks: Map::new(gamedata::GAMEDATA.blocks.keys().collect(), map_size)?,
            entities: Map::new(gamedata::GAMEDATA.entities.keys().collect(), map_size)?,
            spawned_entities: Spawned::new(),
            spawned_players: Spawned::new(),
            rng,
        };
        let terrain_parameters = &gamedata::GAMEDATA.terrain;
        let noise = generate_perlin_noise(map_size, map_size, terrain_parameters.octaves, seed);
        for i in 0..noise.len() {
            let mut level = 0;
            for l in 0..terrain_parameters.heights.len() {
                if terrain_parameters.heights[l] > noise[i] {
                    break;
                }
                level += 1;
            }
            world
                .blocks
                .direct_set(i, Some(&terrain_parameters.blocks[level]))?;
        }

        // go through each block in the map and spawn entities based on the block.
        for y in 0..map_size {
            for x in 0..map_size {
                let block = world
                    .blocks
                    .get(Vector2::new(x, y))?
                    .ok_or_else(|| anyhow!("block doesn't exist at {}, {}", x, y))?;
                let block = gamedata::GAMEDATA
                    .blocks
                    .get(block)
                    .ok_or_else(|| anyhow!("invalid block name '{}' at {},{}", block, x, y))?;
                if world.rng.gen::<f64>() > block.mob_spawn_chance {
                    continue;
                }
                if block.mob_filter.is_empty() {
                    let index = world.entities.index(Vector2::new(x, y))?;
                    world.entities.map[index] = world.rng.gen_range(0, world.entities.max);
                } else {
                    world.entities.set(
                        Vector2::new(x, y),
                        Some(&block.mob_filter[world.rng.gen_range(0, block.mob_filter.len())]),
                    )?;
                }
            }
        }
        Ok(world)
    }

    fn move_entity(&mut self, start: Vector2, end: Vector2) -> Result<()> {
        self.spawned_entities.mov(start, end)?;
        let entity = self.entities.get(start)?.map(|x| x.to_string());
        self.entities.set(start, None)?;
        self.entities.set(end, (&entity).as_ref())?;
        Ok(())
    }

    fn del_entity(&mut self, loc: Vector2) -> Result<()> {
        self.spawned_entities
            .del(self.spawned_entities.get_handle_by_loc(loc)?)?;
        self.entities.set(loc, None)?;
        Ok(())
    }

    fn spawn_entity(&mut self, loc: Vector2) -> Result<()> {
        let entity_name = self
            .entities
            .get(loc)?
            .ok_or_else(|| anyhow!("no entity at location"))?;
        let template = gamedata::GAMEDATA
            .entities
            .get(entity_name)
            .ok_or_else(|| anyhow!("invalid entity name!"))?;
        let entity = template.construct(loc, entity_name, self.rng.gen())?;
        self.spawned_entities.spawn(entity)?;
        Ok(())
    }

    fn set_block(&mut self, loc: Vector2, name: &Option<String>) -> Result<()> {
        self.blocks.set(loc, name.as_ref())
    }

    pub fn handle_world_update(&mut self, update: WorldUpdate) -> Result<()> {
        match update {
            WorldUpdate::WorldEntityUpdate(update) => match update {
                WorldEntityUpdate::Move(update) => {
                    self.mov_entity(update.start, update.end)?;
                }
                WorldEntityUpdate::Del(update) => {
                    self.del_entity(update.loc)?;
                }
                WorldEntityUpdate::Spawn(update) => {
                    self.spawn_entity(update.loc)?;
                }
            },
            WorldUpdate::WorldBlockUpdate(update) => {
                self.set_block(update.loc, &update.blockname)?;
            }
        };
        Ok(())
    }
}
