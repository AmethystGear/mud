use crate::{
    entity::Entity,
    gamedata::{
        block::{Block, PointLight},
        gamedata::{GameData, StructureName},
        terrain::Biome,
    },
    mob::Mob,
    noise,
    rgb::RGB,
    vector3::Vector3,
};
use anyhow::{anyhow, Result};
use bimap::BiMap;
use rand::{prelude::StdRng, Rng, SeedableRng};
use std::collections::{HashMap, HashSet, VecDeque};

pub struct Map<T> {
    pub dim: Vector3,
    map: Vec<T>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct MobU16(pub u16);

impl MobU16 {
    pub fn as_u16(self) -> Option<u16> {
        if self.0 == u16::MAX {
            None
        } else {
            Some(self.0)
        }
    }

    pub fn empty() -> Self {
        MobU16(u16::MAX)
    }
}

impl<T> Map<T>
where
    T: Clone + Eq,
{
    pub fn new(dim: Vector3, default: T) -> Self {
        Map {
            dim,
            map: vec![default; dim.dim() as usize],
        }
    }

    pub fn index(&self, loc: Vector3) -> Result<usize> {
        if loc.x() >= self.dim.x()
            || loc.x() < 0
            || loc.y() >= self.dim.y()
            || loc.y() < 0
            || loc.z() >= self.dim.z()
            || loc.z() < 0
        {
            Err(anyhow!("point {:?} not in map of dim {:?}", loc, self.dim))
        } else {
            Ok(
                loc.z() as usize * self.dim.x() as usize * self.dim.y() as usize
                    + loc.y() as usize * self.dim.x() as usize
                    + loc.x() as usize,
            )
        }
    }

    pub fn index_to_posn(&self, i: usize) -> Vector3 {
        Vector3::new(
            (i % self.dim.x() as usize) as isize,
            ((i / (self.dim.x() as usize)) % self.dim.y() as usize) as isize,
            (i / (self.dim.x() as usize * self.dim.y() as usize)) as isize,
        )
    }

    pub fn direct_set(&mut self, index: usize, val: T) {
        self.map[index] = val;
    }

    pub fn direct_get(&self, index: usize) -> T {
        self.map[index].clone()
    }

    pub fn set(&mut self, loc: Vector3, val: T) -> Result<()> {
        Ok(self.direct_set(self.index(loc)?, val))
    }

    pub fn get(&self, loc: Vector3) -> Result<T> {
        Ok(self.direct_get(self.index(loc)?))
    }
}

struct SpawnedMobs {
    locs: BiMap<Vector3, usize>,
    mobs: HashMap<usize, Mob>,
}

impl SpawnedMobs {
    pub fn new() -> Self {
        SpawnedMobs {
            locs: BiMap::new(),
            mobs: HashMap::new(),
        }
    }
}

impl SpawnedMobs {
    pub fn get_at(&self, loc: Vector3) -> Option<&Mob> {
        self.mobs.get(self.locs.get_by_left(&loc)?)
    }

    pub fn get_at_mut(&mut self, loc: Vector3) -> Option<&mut Mob> {
        self.mobs.get_mut(self.locs.get_by_left(&loc)?)
    }

    pub fn get(&self, id: usize) -> Option<&Mob> {
        self.mobs.get(&id)
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut Mob> {
        self.mobs.get_mut(&id)
    }

    pub fn get_posn(&self, mob: &Mob) -> Option<Vector3> {
        Some(self.locs.get_by_right(&mob.id().id)?.clone())
    }

    pub fn remove_loc(&mut self, loc: Vector3) -> Option<Mob> {
        let (_, id) = self.locs.remove_by_left(&loc)?;
        self.mobs.remove(&id)
    }

    pub fn remove_mob(&mut self, mob: &Mob) -> Option<Mob> {
        self.locs.remove_by_right(&mob.id().id);
        self.mobs.remove(&mob.id().id)
    }

    pub fn insert(&mut self, loc: Vector3, m: Mob) {
        self.locs.insert(loc, m.id().id);
        self.mobs.insert(m.id().id, m);
    }
}

fn get_rand(seed: u64) -> StdRng {
    SeedableRng::seed_from_u64(seed)
}

struct Noise<'a, 'b, 'c> {
    bounding: &'a Vec<f64>,
    terrain: &'b Vec<f64>,
    biome: &'c Vec<f64>,
}

fn generate_biome(
    block_map: &mut Map<u8>,
    _mob_map: &mut Map<MobU16>,
    biome_map: &mut Map<u8>,
    biome: &Biome,
    noise: Noise,
    g: &GameData,
    level: usize,
    cutoff: f64,
) -> Result<()> {
    let start = block_map.index(Vector3::new(0, 0, level as isize))?;
    let layer_size = (block_map.dim.x() * block_map.dim.y()) as usize;

    for i in 0..layer_size {
        if noise.bounding[i] < cutoff {
            let mut block = None;
            for check in &biome.terrain_pass {
                if noise.terrain[i] > check.above {
                    block = Some(&check.then);
                    break;
                }
            }
            if block.is_none() {
                for check in &biome.biome_pass {
                    if noise.biome[i] > check.above {
                        block = Some(&check.then);
                        break;
                    }
                }
            }
            if let Some(name) = block {
                block_map.direct_set(i + start, g.get_block_id_by_blockname(name)?);
                biome_map.direct_set(
                    i + start,
                    g.biomes
                        .id_to_name
                        .get_by_right(&biome.name)
                        .ok_or(anyhow!(format!("invalid biome name {:?}", biome.name)))?
                        .clone(),
                );
            } else {
                return Err(anyhow!(format!(
                    "bad biome specification for {:?}, empty block!",
                    biome.name
                )));
            }
        }
    }
    Ok(())
}

fn generate_structures(
    block_map: &mut Map<u8>,
    mob_map: &mut Map<MobU16>,
    biome_map: &mut Map<u8>,
    g: &GameData,
    rng: &mut StdRng,
) -> Result<()> {
    let mut structure_names: Vec<StructureName> = g.structures.keys().cloned().collect();
    structure_names.sort();
    // generate structures
    for name in &structure_names {
        let structure = g
            .structures
            .get(name)
            .ok_or(anyhow!("invalid structure name"))?;
        if let Some(biome_pairs) = g.terrain.structure_spawn.get(name) {
            for i in 0..(block_map.dim.dim() as usize) {
                let biome_name = g
                    .biomes
                    .id_to_name
                    .get_by_left(&biome_map.direct_get(i))
                    .ok_or(anyhow!("invalid biome id"))?;

                let chance: f64 = rng.gen();
                let mut prob = -1.0;
                for biome_pair in biome_pairs {
                    if &biome_pair.biome == biome_name {
                        prob = biome_pair.prob;
                        break;
                    }
                }

                if chance < prob {
                    let structure = &structure[rng.gen_range(0, structure.len())];
                    // if we can't spawn it here, just move on and ignore the error
                    let _ignore_err =
                        structure.spawn_at(block_map.index_to_posn(i), block_map, mob_map, g, rng);
                }
            }
        } else {
            return Err(anyhow!(format!("invalid structure name {:?}", name)));
        }
    }
    Ok(())
}

fn get_block_by_loc<'a>(block_map: &Map<u8>, g: &'a GameData, loc: Vector3) -> Result<&'a Block> {
    let i = block_map.index(loc)?;
    get_block(block_map, g, i)
}

fn get_block<'a>(block_map: &Map<u8>, g: &'a GameData, i: usize) -> Result<&'a Block> {
    let block = g.get_block_name_by_id(block_map.direct_get(i))?;
    g.blocks
        .name_to_item
        .get(&block)
        .ok_or(anyhow!("block doesn't exist!"))
}

fn expand_point_light(
    light_map: &mut Map<RGB>,
    block_map: &Map<u8>,
    lighting: &PointLight,
    g: &GameData,
    loc: Vector3,
) -> Result<()> {
    let mut visited = HashSet::new();
    let mut to_eval = VecDeque::new();
    to_eval.push_back((loc, 0));
    visited.insert(loc);
    let mut first = true;
    while let Some((curr, depth)) = to_eval.pop_front() {
        let block = get_block_by_loc(block_map, g, curr)?;
        if block.unlit && !first {
            continue;
        } else if !block.unlit {
            let intensity = (lighting.intensity - (depth as f64) * lighting.falloff).max(0.0);
            let color = light_map.get(curr)?;
            light_map.set(curr, color.add(lighting.color.scale(intensity)))?;
        }

        if block.solid && !first {
            continue;
        }
        if depth < lighting.max_range {
            let neighbors = [
                curr - Vector3::new(1, 0, 0),
                curr + Vector3::new(1, 0, 0),
                curr - Vector3::new(0, 1, 0),
                curr + Vector3::new(0, 1, 0),
                curr - Vector3::new(1, 1, 0),
                curr + Vector3::new(1, 1, 0),
                curr - Vector3::new(1, 0, 0) + Vector3::new(0, 1, 0),
                curr + Vector3::new(1, 0, 0) - Vector3::new(0, 1, 0),
            ];
            for neighbor in &neighbors {
                if !visited.contains(neighbor) && block_map.get(*neighbor).is_ok() {
                    to_eval.push_back((neighbor.clone(), depth + 1));
                    visited.insert(*neighbor);
                }
            }
        }
        first = false;
    }
    Ok(())
}

fn gen_noise(rng: &mut StdRng, g: &GameData, biome: bool) -> Vec<f64> {
    noise::generate_perlin_noise(
        g.terrain.dim.x() as usize,
        g.terrain.dim.y() as usize,
        if biome {
            g.terrain.biome_octaves
        } else {
            g.terrain.octaves
        },
        rng,
    )
}

pub struct World {
    spawned_mobs: SpawnedMobs,
    mob_map: Map<MobU16>,
    block_map: Map<u8>,
    light_map: Map<RGB>,
    seed: u64,
    id: usize,
    pub rng: StdRng,
}

impl World {
    pub fn from_seed(seed: u64, g: &GameData) -> Result<World> {
        let mut rng = get_rand(seed);

        // generate blocks
        let mut biome_noise = gen_noise(&mut rng, &g, true);
        let mut terrain_noise = gen_noise(&mut rng, &g, true);
        let mut bounding_noise = gen_noise(&mut rng, &g, false);
        let mut block_map = Map::new(g.terrain.dim, 0u8);
        let mut mob_map = Map::new(g.terrain.dim, MobU16::empty());
        let mut biome_map = Map::new(g.terrain.dim, 0u8);
        for full_pass in &g.terrain.full_passes {
            for level in 0..full_pass.layers.len() {
                for pass in &full_pass.layers[level] {
                    let noise = Noise {
                        biome: &biome_noise,
                        terrain: &terrain_noise,
                        bounding: &bounding_noise,
                    };
                    let biome = g
                        .biomes
                        .name_to_item
                        .get(&pass.biome)
                        .ok_or(anyhow!(format!("{:?} is not a biome", pass.biome)))?;
                    generate_biome(
                        &mut block_map,
                        &mut mob_map,
                        &mut biome_map,
                        biome,
                        noise,
                        g,
                        level,
                        pass.cutoff,
                    )?;
                    biome_noise = gen_noise(&mut rng, &g, true);
                    if full_pass.change_bounding_noise_per_pass {
                        bounding_noise = gen_noise(&mut rng, &g, false);
                    }
                }
                terrain_noise = gen_noise(&mut rng, &g, true);
            }
            bounding_noise = gen_noise(&mut rng, &g, false);
        }
        // generate structures
        generate_structures(&mut block_map, &mut mob_map, &mut biome_map, &g, &mut rng)?;

        // generate mobs
        for i in 0..(mob_map.dim.dim() as usize) {
            let block = g.get_block_name_by_id(block_map.direct_get(i))?;
            let block = g
                .blocks
                .name_to_item
                .get(&block)
                .ok_or(anyhow!("block doesn't exist!"))?;
            if rng.gen::<f64>() < block.mob_spawn_chance && mob_map.direct_get(i) == MobU16::empty()
            {
                mob_map.direct_set(
                    i,
                    MobU16(rng.gen_range(0, g.mob_templates.max_id.as_u16().unwrap_or(0))),
                );
            }
        }

        // calculate lighting
        let mut light_map = Map::new(g.terrain.dim, RGB::new(0, 0, 0));

        // sunlight
        for i in 0..(light_map.dim.dim() as usize) {
            let block = get_block(&block_map, g, i)?;
            if block.unlit {
                light_map.direct_set(i, RGB::white());
                continue;
            }
            let mut light = RGB::white();
            let loc = block_map.index_to_posn(i);
            for z in 0..loc.z() {
                let loc = Vector3::new(loc.x(), loc.y(), z);
                let block = get_block_by_loc(&block_map, g, loc)?;
                light = light.mul(block.transparency);
            }
            light_map.direct_set(i, light);
        }

        // light emitters
        for i in 0..(light_map.dim.dim() as usize) {
            let block = get_block(&block_map, g, i)?;
            // expand each point light individually
            if let Some(lighting) = &block.light.point_light {
                expand_point_light(
                    &mut light_map,
                    &block_map,
                    lighting,
                    g,
                    block_map.index_to_posn(i),
                )?;
            }
            // expand down lights (assuming there's a block below this one)
            if let Some(lighting) = &block.light.down_light {
                let loc = block_map.index_to_posn(i) + Vector3::new(0, 0, 1);
                if let Ok(below) = get_block_by_loc(&block_map, g, loc) {
                    if !below.unlit {
                        let color = light_map.get(loc)?;
                        light_map.set(loc, color.add(lighting.color.scale(lighting.intensity)))?;
                    }
                }
            }
        }

        let min_light = 30;

        for i in 0..(light_map.dim.dim() as usize) {
            let block = get_block(&block_map, g, i)?;
            let mut light = light_map.direct_get(i);
            if light.r < min_light {
                light.r = min_light;
            }
            if light.g < min_light {
                light.g = min_light;
            }
            if light.b < min_light {
                light.b = min_light;
            }
            light_map.direct_set(i, light.mul(block.color));
        }

        Ok(World {
            spawned_mobs: SpawnedMobs::new(),
            mob_map,
            block_map,
            light_map,
            seed,
            rng,
            id: 0,
        })
    }

    fn get_spawned_mob_at(&self, loc: Vector3) -> Result<&Mob> {
        if let Some(mob) = self.spawned_mobs.get_at(loc) {
            Ok(mob)
        } else {
            Err(anyhow!(format!("no mob at location {:?}", loc)))
        }
    }

    fn get_spawned_mob_at_mut(&mut self, loc: Vector3) -> Result<&mut Mob> {
        if let Some(mob) = self.spawned_mobs.get_at_mut(loc) {
            Ok(mob)
        } else {
            Err(anyhow!(format!("no mob at location {:?}", loc)))
        }
    }

    pub fn has_mob(&self, loc: Vector3) -> Result<bool> {
        Ok(self.mob_map.get(loc)?.as_u16().is_some())
    }

    pub fn get_mob_at_mut(&mut self, loc: Vector3, g: &GameData) -> Result<&mut Mob> {
        // why not if let here?
        // because that would make rust complain about self.spawn_mob
        if self.spawned_mobs.get_at(loc).is_some() {
            Ok(self.spawned_mobs.get_at_mut(loc).unwrap())
        } else {
            if self.mob_map.get(loc)?.as_u16().is_some() {
                self.spawn_mob(loc, g)?;
                self.get_spawned_mob_at_mut(loc)
            } else {
                Err(anyhow!(format!("no mob at location {:?}", loc)))
            }
        }
    }

    pub fn get_mob(&self, id: usize) -> Result<&Mob> {
        if let Some(mob) = self.spawned_mobs.get(id) {
            Ok(mob)
        } else {
            Err(anyhow!(format!("no mob with id {:?}", id)))
        }
    }

    pub fn get_mob_mut(&mut self, id: usize) -> Result<&mut Mob> {
        if let Some(mob) = self.spawned_mobs.get_mut(id) {
            Ok(mob)
        } else {
            Err(anyhow!(format!("no mob with id {:?}", id)))
        }
    }

    pub fn delete_mob_by_loc(&mut self, loc: Vector3) -> Result<()> {
        self.mob_map.set(loc, MobU16::empty())?;
        self.spawned_mobs.remove_loc(loc);
        Ok(())
    }

    pub fn delete_mob(&mut self, mob: Mob) -> Result<()> {
        let loc = self
            .spawned_mobs
            .get_posn(&mob)
            .ok_or_else(|| anyhow!(format!("the mob {:?} doesn't exist", mob)))?;
        self.delete_mob_by_loc(loc)
    }

    pub fn move_mob(&mut self, start: Vector3, end: Vector3) -> Result<()> {
        if let Some(_) = self.mob_map.get(end)?.as_u16() {
            return Err(anyhow!(format!("there's already a mob at {:?}", end)));
        }
        if let Some(mob) = self.spawned_mobs.remove_loc(start) {
            self.spawned_mobs.insert(end, mob);
        }
        let val = self.mob_map.get(start)?;
        self.mob_map.set(start, MobU16::empty())?;
        self.mob_map.set(end, val)?;
        Ok(())
    }

    fn spawn_mob(&mut self, loc: Vector3, g: &GameData) -> Result<()> {
        let mob_name = g.get_mob_name_by_id(self.mob_map.get(loc)?)?;
        let mob_template = g
            .mob_templates
            .name_to_item
            .get(&mob_name)
            .ok_or_else(|| anyhow!("invalid mob name?"))?;
        self.spawned_mobs
            .insert(loc, Mob::new(self.id, loc, mob_template, &mut self.rng, g)?);
        self.id += 1;
        Ok(())
    }

    pub fn colors(&self) -> &Map<RGB> {
        &self.light_map
    }

    pub fn blocks(&self) -> &Map<u8> {
        &self.block_map
    }

    pub fn mobs(&self) -> &Map<MobU16> {
        &self.mob_map
    }

    pub fn get_block<'a>(&self, g: &'a GameData, i: usize) -> Result<&'a Block> {
        let block = g.get_block_name_by_id(self.block_map.direct_get(i))?;
        g.blocks
            .name_to_item
            .get(&block)
            .ok_or(anyhow!("block doesn't exist!"))
    }

    pub fn get_block_at<'a>(&self, g: &'a GameData, loc: Vector3) -> Result<&'a Block> {
        self.get_block(g, self.blocks().index(loc)?)
    }
}
