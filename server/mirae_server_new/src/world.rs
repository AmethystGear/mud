use crate::{
    gamedata::{
        block::{Block, Lighting},
        gamedata::GameData,
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
            map: vec![default; dim.dim()],
        }
    }

    pub fn index(&self, loc: Vector3) -> Result<usize> {
        if loc.x() >= self.dim.x() || loc.y() >= self.dim.y() || loc.z() >= self.dim.z() {
            Err(anyhow!("point {:?} not in map of dim {:?}", loc, self.dim))
        } else {
            Ok(loc.z() * self.dim.x() * self.dim.y() + loc.y() * self.dim.x() + loc.x())
        }
    }

    pub fn index_to_posn(&self, i: usize) -> Vector3 {
        Vector3::new(
            i % self.dim.x(),
            (i / (self.dim.x())) % self.dim.y(),
            i / (self.dim.x() * self.dim.y()),
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
    locs: BiMap<Vector3, u64>,
    mobs: HashMap<u64, Mob>,
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
    pub fn get(&self, loc: Vector3) -> Option<&Mob> {
        self.mobs.get(self.locs.get_by_left(&loc)?)
    }

    pub fn get_posn(&self, mob: &Mob) -> Option<Vector3> {
        Some(self.locs.get_by_right(&mob.id())?.clone())
    }

    pub fn remove_loc(&mut self, loc: Vector3) -> Option<Mob> {
        let (_, id) = self.locs.remove_by_left(&loc)?;
        self.mobs.remove(&id)
    }

    pub fn remove_mob(&mut self, mob: &Mob) -> Option<Mob> {
        self.locs.remove_by_right(&mob.id());
        self.mobs.remove(&mob.id())
    }

    pub fn insert(&mut self, loc: Vector3, m: Mob) {
        self.locs.insert(loc, m.id());
        self.mobs.insert(m.id(), m);
    }
}

fn get_rand(seed: u64) -> StdRng {
    let bytes = seed.to_le_bytes();
    let mut seed_bytes = [0; 32];
    for i in 0..8 {
        seed_bytes[i] = bytes[i];
    }

    SeedableRng::from_seed(seed_bytes)
}

struct Noise<'a, 'b, 'c> {
    bounding: &'a Vec<f64>,
    terrain: &'b Vec<f64>,
    biome: &'c Vec<f64>,
}

fn generate_biome(
    block_map: &mut Map<u8>,
    mob_map: &mut Map<MobU16>,
    structure_map: &mut Map<bool>,
    biome: &Biome,
    noise: Noise,
    g: &GameData,
    level: usize,
    cutoff: f64,
    rng: &mut StdRng,
) -> Result<()> {
    let start = block_map.index(Vector3::new(0, 0, level))?;
    let layer_size = block_map.dim.x() * block_map.dim.y();
    // generate terrain
    for i in 0..layer_size {
        if structure_map.direct_get(i + start) {
            continue;
        }
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
            } else {
                return Err(anyhow!(format!(
                    "bad biome specification for {:?}, empty block!",
                    biome.name
                )));
            }
        }
    }
    // generate structures
    for i in 0..layer_size {
        if noise.bounding[i] < cutoff {
            for structure in &biome.spawn {
                let chance: f64 = rng.gen();
                if chance < structure.prob {
                    let structure = g
                        .structures
                        .get(&structure.structure)
                        .ok_or(anyhow!(format!(
                            "invalid structure name {:?}",
                            structure.structure
                        )))?;

                    let structure = &structure[rng.gen_range(0, structure.len())];
                    // if we can't spawn it here, just move on and ignore the error
                    let _ = structure.spawn_at(
                        block_map.index_to_posn(start + i),
                        block_map,
                        mob_map,
                        structure_map,
                        g,
                        rng,
                    );
                    break;
                }
            }
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
    g.blocks.get(&block).ok_or(anyhow!("block doesn't exist!"))
}

fn expand_light(
    light_map: &mut Map<RGB>,
    block_map: &Map<u8>,
    lighting: &Lighting,
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

pub struct World {
    spawned_mobs: SpawnedMobs,
    mob_map: Map<MobU16>,
    block_map: Map<u8>,
    light_map: Map<RGB>,
    seed: u64,
    id: u64,
    rng: StdRng,
}

impl World {
    pub fn from_seed(seed: u64, g: &GameData) -> Result<World> {
        let mut rng = get_rand(seed);

        // make a closure that will return us new noise every time we call it
        let gen_noise = |seed| {
            let mut rng = get_rand(seed);
            noise::generate_perlin_noise(
                g.terrain.dim.x(),
                g.terrain.dim.y(),
                g.terrain.octaves,
                &mut rng,
            )
        };
        // generate blocks
        let mut biome_noise = gen_noise(rng.gen());
        let mut terrain_noise = gen_noise(rng.gen());
        let mut bounding_noise = gen_noise(rng.gen());
        let mut block_map = Map::new(g.terrain.dim, 0u8);
        let mut mob_map = Map::new(g.terrain.dim, MobU16::empty());
        let mut structure_map = Map::new(g.terrain.dim, false);
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
                        .get(&pass.biome)
                        .ok_or(anyhow!(format!("{:?} is not a biome", pass.biome)))?;
                    generate_biome(
                        &mut block_map,
                        &mut mob_map,
                        &mut structure_map,
                        biome,
                        noise,
                        g,
                        level,
                        pass.cutoff,
                        &mut get_rand(rng.gen()),
                    )?;
                    biome_noise = gen_noise(rng.gen());
                    if full_pass.change_bounding_noise_per_pass {
                        bounding_noise = gen_noise(rng.gen());
                    }
                }
                terrain_noise = gen_noise(rng.gen());
            }
            bounding_noise = gen_noise(rng.gen());
        }

        // generate mobs
        for i in 0..(mob_map.dim.dim()) {
            let block = g.get_block_name_by_id(block_map.direct_get(i))?;
            let block = g
                .blocks
                .get(&block)
                .ok_or(anyhow!("block doesn't exist!"))?;
            if rng.gen::<f64>() < block.mob_spawn_chance && mob_map.direct_get(i) == MobU16::empty()
            {
                mob_map.direct_set(
                    i,
                    MobU16(rng.gen_range(0, g.max_mob_id.as_u16().unwrap_or(0))),
                );
            }
        }

        // calculate lighting
        let mut light_map = Map::new(g.terrain.dim, RGB::new(0, 0, 0));

        // sunlight
        for i in 0..(light_map.dim.dim()) {
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
        for i in 0..(light_map.dim.dim()) {
            let block = get_block(&block_map, g, i)?;
            if let Some(lighting) = &block.light {
                expand_light(
                    &mut light_map,
                    &block_map,
                    lighting,
                    g,
                    block_map.index_to_posn(i),
                )?;
            }
        }

        let min_light = 30;

        for i in 0..(light_map.dim.dim()) {
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

    pub fn get_mob(&mut self, loc: Vector3, g: &GameData) -> Result<Mob> {
        if let Some(mob) = self.spawned_mobs.get(loc) {
            Ok(mob.clone())
        } else if let Some(_) = self.mob_map.get(loc)?.as_u16() {
            self.spawn_mob(loc, g)?;
            self.get_mob(loc, g)
        } else {
            Err(anyhow!(format!("no mob at location {:?}", loc)))
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
}
