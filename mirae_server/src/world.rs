extern crate rand;

use crate::perlin_noise;
use crate::scanner;
use crate::stats;
use char_stream::CharStream;
use rand::Rng;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::u16;

pub const MAP_SIZE: u16 = 200;
const ENTITIES_CONFIG_DIR: &str = "config/instantiables/";
const ITEMS_CONFIG: &str = "config/items.txt";
const TERRAIN_CONFIG: &str = "config/terrain.txt";

struct Map {
    id_to_name: HashMap<u16, String>,
    name_to_stats: HashMap<String, stats::Stats>,
    map: Vec<u16>,
}

impl Map {
    pub fn new() -> Self {
        Map {
            id_to_name: HashMap::new(),
            name_to_stats: HashMap::new(),
            map: vec![u16::MAX; (MAP_SIZE as usize) * (MAP_SIZE as usize)],
        }
    }
}

pub struct World {
    blocks: Map,
    entities: Map,
    items: stats::Stats,
    items_tiered: HashMap<i64, Vec<String>>,
    max_entity_id: u16,
    max_block_id: u16,
    seed: i64,
}

impl World {
    pub fn items(&self) -> stats::Stats {
        return self.items.clone();
    }

    pub fn items_tiered(&self) -> &HashMap<i64, Vec<String>> {
        return &self.items_tiered;
    }

    pub fn max_entity_id(&self) -> u16 {
        return self.max_entity_id;
    }

    pub fn max_block_id(&self) -> u16 {
        return self.max_block_id;
    }
}

fn get_blocks(
    world: &mut World,
    terrain_configuration: &stats::Stats,
    add_ids: bool,
) -> Result<i64, Box<dyn Error>> {
    let mut blocks = stats::get(terrain_configuration, "blocks")?.as_box()?;
    let block_names = stats::get_var_names(&blocks);

    let mut last_id = -1;
    if add_ids {
        stats::add_ids_to_boxes(&mut blocks, 0);
    }
    for block_name in block_names {
        let block = stats::get(&blocks, block_name.as_str())?.as_box()?;
        let id = stats::get(&block, "id")?.as_int()?;
        if last_id < id {
            last_id = id;
        }
        world
            .blocks
            .id_to_name
            .insert(id as u16, block_name.clone());
        world.blocks.name_to_stats.insert(block_name, block);
    }
    return Ok(last_id + 1);
}

fn get_entities(
    world: &mut World,
    entity_config: &mut stats::Stats,
    f_name: Option<String>,
    id_start: Option<i64>,
) -> Result<i64, Box<dyn Error>> {
    let mut last_id = -1;
    if let Some(id) = id_start {
        stats::add_ids_to_boxes(entity_config, id);
    }
    let entity_names = stats::get_var_names(&entity_config);
    for entity_name in entity_names {
        let mut entity = stats::get(&entity_config, entity_name.as_str())?.as_box()?;
        if !stats::has_var(&entity, "entity_type") {
            if let Some(fname) = f_name.clone() {
                stats::set(
                    &mut entity,
                    "entity_type",
                    stats::Value::String(fname.clone()),
                );
            }
        }
        let id = stats::get(&entity, "id")?.as_int()?;
        if id > last_id {
            last_id = id;
        }
        world
            .entities
            .id_to_name
            .insert(id as u16, entity_name.clone());
        world.entities.name_to_stats.insert(entity_name, entity);
    }
    return Ok(last_id + 1);
}

fn tier_items(world: &mut World) -> Result<(), Box<dyn Error>> {
    let item_names = stats::get_var_names(&world.items());
    for item in item_names {
        let spawn = stats::get_or_else(
            &stats::get(&world.items, &item)?.as_box()?,
            "spawn",
            &stats::Value::Box(stats::Stats::new()),
        )
        .as_box()?;
        let tier = stats::get_or_else(&spawn, "tier", &stats::Value::Int(-1)).as_int()?;
        if world.items_tiered.contains_key(&tier) {
            world
                .items_tiered
                .get_mut(&tier)
                .ok_or("this should never happen")?
                .push(item);
        } else {
            world.items_tiered.insert(tier, vec![item]);
        }
    }
    return Ok(());
}

pub fn from_seed(seed: i64) -> Result<World, Box<dyn Error>> {
    let mut world = World {
        blocks: Map::new(),
        entities: Map::new(),
        items: stats::Stats::new(),
        max_entity_id: 0,
        max_block_id: 0,
        seed,
        items_tiered: HashMap::new(),
    };
    world.items = stats::from(&mut scanner::from(CharStream::from_file(File::open(
        ITEMS_CONFIG,
    )?)))?;

    tier_items(&mut world)?;

    let terrain_configuration = stats::from(&mut scanner::from(CharStream::from_file(
        File::open(TERRAIN_CONFIG)?,
    )))?;

    world.max_block_id = get_blocks(&mut world, &terrain_configuration, true)? as u16;

    // generate terrain based on parameters provided
    let terrain_params = stats::get(&terrain_configuration, "terrain_parameters")?.as_box()?;
    let octaves = stats::get(&terrain_params, "octaves")?.as_int()?;
    let perlin_noise =
        perlin_noise::generate_perlin_noise(MAP_SIZE, MAP_SIZE, octaves as u8, world.seed);

    let height_map = stats::get(&terrain_params, "height_map")?.as_box()?;
    let height_blocks = stats::get(&height_map, "blocks")?.as_vec()?;
    let heights = stats::get(&height_map, "heights")?.as_vec()?;
    for i in 0..perlin_noise.len() {
        let mut level = 0;
        for l in 0..heights.len() {
            if heights[l].as_flt()? > perlin_noise[i] {
                break;
            }
            level += 1;
        }

        let block = world
            .blocks
            .name_to_stats
            .get(&height_blocks[level].as_string()?)
            .ok_or(format!("no block at level {}", level))?;
        world.blocks.map[i] = stats::get(block, "id")?.as_int()? as u16;
    }

    let files = fs::read_dir(ENTITIES_CONFIG_DIR)?;
    let mut last_id = 0;
    for file in files {
        let file_uw = file?.path();
        let f_name = file_uw
            .file_name()
            .ok_or("error getting file name")?
            .to_str()
            .ok_or("error getting file name")?;
        let entity_config = File::open(file_uw.clone())?;
        let mut f_entities = stats::from(&mut scanner::from(CharStream::from_file(entity_config)))?;
        last_id = get_entities(
            &mut world,
            &mut f_entities,
            Some(f_name.to_string()),
            Some(last_id),
        )?;
    }
    world.max_entity_id = last_id as u16;

    let mut rng = rand::thread_rng();
    for y in 0..MAP_SIZE {
        for x in 0..MAP_SIZE {
            let block: &stats::Stats = get_block(&world, x, y)?;
            if stats::has_var(block, "mob_spawn_chance") {
                let spawn_chance = stats::get(block, "mob_spawn_chance")?.as_flt()?;
                if rng.gen::<f64>() < spawn_chance {
                    world.entities.map[index(x, y) as usize] = rng.gen_range(0, last_id) as u16;
                }
            }
        }
    }
    return Ok(world);
}

pub fn from_save(file: File) -> Result<World, Box<dyn Error>> {
    let mut world = World {
        blocks: Map::new(),
        entities: Map::new(),
        items: stats::Stats::new(),
        max_entity_id: 0,
        max_block_id: 0,
        seed: 0,
        items_tiered: HashMap::new(),
    };
    let stats = stats::from(&mut scanner::from(CharStream::from_file(file)))?;
    world.items = stats::from(&mut scanner::from(CharStream::from_file(File::open(
        ITEMS_CONFIG,
    )?)))?;
    tier_items(&mut world)?;
    // get terrain from stats
    let terrain = stats::get(&stats, "terrain")?.as_box()?;
    world.max_block_id = get_blocks(&mut world, &terrain, false)? as u16;
    let block_map = stats::get(&terrain, "block_map")?.as_vec()?;
    let entity_map = stats::get(&terrain, "entity_map")?.as_vec()?;
    for i in 0..block_map.len() {
        world.blocks.map[i] = block_map[i].as_int()? as u16;
        world.entities.map[i] = entity_map[i].as_int()? as u16;
    }

    // get seed from stats
    world.seed = stats::get(&stats, "seed")?.as_int()?;

    let mut entities = stats::get(&stats, "entities")?.as_box()?;
    world.max_entity_id = get_entities(&mut world, &mut entities, None, None)? as u16;
    return Ok(world);
}

pub fn save_to(world: &World, file: File) -> Result<(), Box<dyn Error>> {
    let mut stats = stats::Stats::new();
    // add seed to stats
    stats::set(&mut stats, "seed", stats::Value::Int(world.seed));

    // create and add terrain to stats
    let mut terrain = stats::Stats::new();
    let mut block_map = Vec::new();
    let mut entity_map = Vec::new();
    for i in 0..world.blocks.map.len() {
        block_map.push(stats::Value::Int(world.blocks.map[i] as i64));
        entity_map.push(stats::Value::Int(world.entities.map[i] as i64));
    }
    stats::set(&mut terrain, "block_map", stats::Value::List(block_map));
    stats::set(&mut terrain, "entity_map", stats::Value::List(entity_map));

    let mut blocks = stats::Stats::new();
    for (name, block) in world.blocks.name_to_stats.clone() {
        stats::set(&mut blocks, name.as_str(), stats::Value::Box(block));
    }
    stats::set(&mut terrain, "blocks", stats::Value::Box(blocks));
    stats::set(&mut stats, "terrain", stats::Value::Box(terrain));

    let mut entities = stats::Stats::new();
    for (name, entity) in world.blocks.name_to_stats.clone() {
        stats::set(&mut entities, name.as_str(), stats::Value::Box(entity));
    }
    stats::set(&mut stats, "entities", stats::Value::Box(entities));
    return stats::save_to(&stats, file);
}

fn index(x: u16, y: u16) -> usize {
    return (y as usize) * (MAP_SIZE as usize) + (x as usize);
}

pub fn get_block_id(world: &World, x: u16, y: u16) -> u16 {
    return world.blocks.map[index(x, y)];
}

pub fn get_entity_id(world: &World, x: u16, y: u16) -> u16 {
    return world.entities.map[index(x, y)];
}

pub fn get_block(world: &World, x: u16, y: u16) -> Result<&stats::Stats, Box<dyn Error>> {
    let block_name = world
        .blocks
        .id_to_name
        .get(&world.blocks.map[index(x, y) as usize])
        .ok_or("no block exists with this ID!")?;
    return Ok(world
        .blocks
        .name_to_stats
        .get(&block_name.clone())
        .ok_or(format!("no block exists with the name {}", block_name))?);
}

pub fn get_block_by_id(world: &World, id: u16) -> Result<&stats::Stats, Box<dyn Error>> {
    return Ok(world
        .blocks
        .name_to_stats
        .get(
            world
                .blocks
                .id_to_name
                .get(&id)
                .ok_or("no block with the provided id")?,
        )
        .ok_or("no block with the provided name")?);
}

pub fn get_entity_name(world: &World, x: u16, y: u16) -> Option<String> {
    let name = world
        .entities
        .id_to_name
        .get(&world.entities.map[index(x, y) as usize]);
    if name.is_none() {
        return None;
    } else {
        return Some(name?.clone());
    }
}

pub fn get_entity_properties(world: &World, x: u16, y: u16) -> Option<&stats::Stats> {
    let entity_name = get_entity_name(world, x, y);
    return world.entities.name_to_stats.get(&entity_name?.clone());
}

pub fn get_entity_properties_by_id(
    world: &World,
    id: u16,
) -> Result<&stats::Stats, Box<dyn Error>> {
    return Ok(world
        .entities
        .name_to_stats
        .get(
            world
                .entities
                .id_to_name
                .get(&id)
                .ok_or("no entity with the provided id")?,
        )
        .ok_or("no entity with the provided name")?);
}

pub fn has_entity(world: &World, x: u16, y: u16) -> bool {
    return world.entities.map[index(x, y)] != u16::MAX;
}

pub fn remove_entity(world: &mut World, x: u16, y: u16) {
    world.entities.map[index(x, y)] = u16::MAX;
}
