use std::{fmt::Debug, collections::HashMap};
use anyhow::{Result, anyhow};
use crate::world::perlin_noise::generate_perlin_noise;
use crate::deser::{entity::EntityTemplate, block::Block, gamemode::GameMode};
use crate::requests::worldupdate::{EntityUpdate, BlockUpdate, EntityMove};
use rand::Rng;
use super::spawned_entities::SpawnedEntities;

struct Map<T> where T : Debug {
    id_to_name: HashMap<u16, String>,
    name_to_id: HashMap<String, u16>,
    name_to_item: HashMap<String, T>,
    map_size: u16,
    map: Vec<u16>,
    max : u16
}

impl <T : Debug> Map<T> {
    fn new(name_to_item : HashMap<String, T>, map_size : u16) -> Self {
        let mut id_to_name = HashMap::new();
        let mut name_to_id = HashMap::new();
        let mut id : u16 = 0;
        for (k, v) in &name_to_item {
            name_to_id.insert(k.clone(), id);
            id_to_name.insert(id, k.clone());
            id += 1;
        }
        Map {
            id_to_name,
            name_to_id,
            name_to_item,
            map_size,
            map: vec![u16::MAX; (map_size as usize) * (map_size as usize)],
            max: id
        }
    }

    fn index(&self, x : u16, y : u16) -> Result<usize> {
        if x >= self.map_size || y >= self.map_size {
            Err(anyhow!("point ({},{}) not in map of size {}", x, y, self.map_size))
        } else {
            Ok((y as usize) * (self.map_size as usize)  + (x as usize))
        }
    }

    fn set(&mut self, x : u16, y : u16, name : &str) -> Result<()> {
        let index = self.index(x, y)?;
        self.direct_set(index, name)
    }

    fn direct_set(&mut self, index : usize, name : &str) -> Result<()> {
        if let Some(id) = self.name_to_id.get(name) {
            self.map[index] = id.clone();
            Ok(())
        } else {
            Err(anyhow!("object with name {} does not exist in map {:?}", name, self.name_to_id))
        }
    }

    fn get_name(&self, x : u16, y : u16) -> Result<Option<&String>> {
        let index = self.index(x, y)?;
        if self.map[index] == u16::MAX {
            Ok(None)
        } else {
            if let Some(name) = self.id_to_name.get(&self.map[index]) { 
                Ok(Some(name))
            } else {
                Err(anyhow!("the id {} at ({}, {}), doesn't have a mapping in {:?}", self.map[index], x, y, self.id_to_name))
            }
        }
    }

    fn get_item(&self, x : u16, y : u16) -> Result<Option<&T>> {
        let name = self.get_name(x, y)?;
        if let Some(name) = name {
            if let Some(item) = self.name_to_item.get(name) {
                Ok(Some(item))
            } else {
                Err(anyhow!("the entity named {} at ({}, {}) doesn't have a mapping in {:?}", name, x, y, self.name_to_item))
            }
        } else {
            Ok(None)
        }
    }
}

pub struct World {
    blocks: Map<Block>,
    entities: Map<EntityTemplate>,
    spawned_entities: SpawnedEntities,
    seed : i64
}

impl World {
    pub fn from_seed(gamemode : GameMode, seed : i64) -> Result<Self> {
        let map_size = gamemode.terrain_parameters.map_size;
        let mut world = World {
            blocks: Map::new(gamemode.blocks, map_size),
            entities: Map::new(gamemode.entities, map_size),
            spawned_entities: SpawnedEntities::new(),
            seed
        };
        let terrain_parameters = gamemode.terrain_parameters;
        let noise = generate_perlin_noise(map_size, map_size, terrain_parameters.octaves, world.seed);
        for i in 0..noise.len() {
            let mut level = 0;
            for l in 0..terrain_parameters.heights.len() {
                if terrain_parameters.heights[l] > noise[i] {
                    break;
                }
                level += 1;
            }
            world.blocks.direct_set(i, &terrain_parameters.blocks[level]);
        }

        let rng = rand::thread_rng();
        for y in 0..map_size {
            for x in 0..map_size {
                let block = world.blocks.get_item(x, y)?.ok_or_else(|| anyhow!("block doesn't exist at {}, {}", x, y))?;
                if rng.gen::<f64>() > block.mob_spawn_chance {
                    continue;
                }
                if block.mob_filter.is_empty() {
                    world.entities.map[world.entities.index(x, y)?] = rng.gen_range(0, world.entities.max);
                } else {
                    world.entities.set(x, y, &block.mob_filter[rng.gen_range(0, block.mob_filter.len())]);
                }
            }
        }

        Ok(world)
    }

    pub fn update_block(&mut self, update : BlockUpdate) -> Result<()> {
        self.blocks.set(update.x, update.y, &update.blockname)
    }

    pub fn update_entity(&mut self, update : EntityUpdate) -> Result<()> {
        match update {
            EntityUpdate::Move(m) => {
                return self.move_entity(m);
            },
            EntityUpdate::Del(d) => {
                return self.del_entity(d);
            },
            EntityUpdate::Spawn(s) => {
                return self.spawn_entity(s);
            }
        }
    }

    pub fn move_entity(&mut self, update : EntityMove) -> Result<()> {
        if let Some(entity) = self.entities.get_item(update.x_start, update.y_start)? {
            let entity = self.spawned_entities.get(update.x_start, update.y_start);
            entity.set_posn(update.x_to, update.y_to);
        } else {
            return Err(anyhow!("no entity at {}, {}", update.x_start, update.y_start));
        }
        Ok(())
    }
}