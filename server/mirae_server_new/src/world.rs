use crate::{gamedata::GameData, mob::Mob, noise, rgb::RGB, terrain::Biome, vector3::Vector3};
use anyhow::{anyhow, Result};
use bimap::BiMap;
use rand::{prelude::StdRng, SeedableRng};
use std::collections::HashMap;

pub struct Map<T> {
    dim: Vector3,
    map: Vec<T>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct MobU16(u16);

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
    T: Clone + Copy + Eq,
{
    pub fn new(dim: Vector3, default: T) -> Self {
        Map {
            dim,
            map: vec![default; dim.x() * dim.y() * dim.z()],
        }
    }

    pub fn index(&self, loc: Vector3) -> Result<usize> {
        if loc.x() >= self.dim.x() || loc.y() >= self.dim.y() || loc.z() >= self.dim.z() {
            Err(anyhow!("point {:?} not in map of dim {:?}", loc, self.dim))
        } else {
            Ok(loc.z() * self.dim.x() * self.dim.y() + loc.y() * self.dim.x() + loc.x())
        }
    }

    pub fn direct_set(&mut self, index: usize, val: T) {
        self.map[index] = val;
    }

    pub fn direct_get(&self, index: usize) -> T {
        self.map[index]
    }

    pub fn set(&mut self, loc: Vector3, val: T) -> Result<()> {
        Ok(self.direct_set(self.index(loc)?, val))
    }

    pub fn get(&mut self, loc: Vector3) -> Result<T> {
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

pub struct World {
    spawned_mobs: SpawnedMobs,
    mob_map: Map<MobU16>,
    block_map: Map<u8>,
    color_map: Map<RGB>,
    id: u64,
}

#[derive(Copy, Clone)]
enum BiomeType {
    Surface,
    Ocean,
    Underwater,
    Underground,
    Bottomground,
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

//TODO: structure generation
fn generate_biome(
    blocks: &mut Map<u8>,
    biome: &Biome,
    noise: Noise,
    g: &GameData,
    level: usize,
    cutoff: f64,
) -> Result<()> {
    let start = blocks.index(Vector3::new(0, 0, level))?;
    let layer_size = blocks.dim.x() * blocks.dim.y();
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
                blocks.direct_set(i + start, g.get_block_id_by_blockname(name)?);
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

impl World {
    pub fn from_seed(seed: u64, g: &GameData) -> Result<World> {
        let mut world = World {
            spawned_mobs: SpawnedMobs::new(),
            mob_map: Map::new(g.terrain.dim, MobU16::empty()),
            block_map: Map::new(g.terrain.dim, 0),
            color_map: Map::new(g.terrain.dim, RGB::new(0, 0, 0)),
            id: 0,
        };
        let mut rng = get_rand(seed);

        let mut gen_noise = || {
            noise::generate_perlin_noise(
                g.terrain.dim.x(),
                g.terrain.dim.y(),
                g.terrain.octaves,
                &mut rng,
            )
        };

        let mut biome_noise = gen_noise();
        let mut terrain_noise = gen_noise();
        let mut bounding_noise = gen_noise();

        let mut blocks = Map::new(g.terrain.dim, 0u8);
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
                    generate_biome(&mut blocks, biome, noise, g, level, pass.cutoff)?;
                    biome_noise = gen_noise();
                    if full_pass.change_bounding_noise_per_pass {
                        bounding_noise = gen_noise();
                    }
                }
                terrain_noise = gen_noise();
            }
        }
        return Ok(world);
    }

    pub fn get_mob(&mut self, loc: Vector3, g: &GameData) -> Result<Mob> {
        if let Some(mob) = self.spawned_mobs.get(loc) {
            Ok(mob.clone())
        } else if let Some(_) = self.mob_map.get(loc)?.as_u16() {
            self.spawn_mob(loc, g)
        } else {
            Err(anyhow!(format!("no mob at location {:?}", loc)))
        }
    }

    pub fn update_mob(&mut self, loc: Vector3, m: Mob) -> Result<()> {
        if let Some(mob) = self.spawned_mobs.get(loc) {
            if mob.id() == m.id() {
                self.spawned_mobs.insert(loc, m);
                Ok(())
            } else {
                Err(anyhow!(format!("the mob you are trying to update {:?} and the mob you provided {:?} aren't the same", mob, m)))
            }
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

    fn spawn_mob(&mut self, loc: Vector3, g: &GameData) -> Result<Mob> {
        let mob_name = g.get_mob_name_by_id(
            self.mob_map
                .get(loc)?
                .as_u16()
                .ok_or_else(|| anyhow!(format!("no mob in location {:?}", loc)))?,
        )?;
        let mob_template = g
            .mob_templates
            .get(&mob_name)
            .ok_or_else(|| anyhow!("invalid mob name?"))?;
        let mob = mob_template.spawn(self.id);
        self.id += 1;
        self.spawned_mobs.insert(loc, mob.clone());
        Ok(mob)
    }
}

/*

#[derive(Debug)]
pub struct Map<V>
where
    V: Eq + Hash + Debug,
{
    id_val_map: BiMap<u16, V>,
    dim: Vector3,
    map: Vec<u16>,
    max: u16,
}

impl<V> Map<V>
where
    V: Eq + Hash + Debug,
{
    fn new(vals: HashSet<V>, dim: Vector3) -> Result<Self> {
        let mut id_val_map = BiMap::new();
        let mut id: u16 = 0;
        for v in vals {
            if id == u16::MAX {
                return Err(anyhow!("+"));
            }
            id_val_map.insert(id, v);
            id += 1;
        }
        Ok(Map {
            id_val_map,
            dim,
            map: vec![u16::MAX; (dim.x as usize) * (dim.y as usize) * (dim.z as usize)],
            max: id,
        })
    }

    fn index(&self, loc: Vector3) -> Result<usize> {
        if loc.x >= self.dim.x || loc.y >= self.dim.y || loc.z >= self.dim.z {
            Err(anyhow!("point {:?} not in map of dim {:?}", loc, self.dim))
        } else {
            Ok(loc.z * self.dim.x * self.dim.y + loc.y * self.dim.x + loc.x)
        }
    }

    fn set(&mut self, loc: Vector3, name: Option<&V>) -> Result<()> {
        let index = self.index(loc)?;
        self.direct_set(index, name)
    }

    fn direct_set(&mut self, index: usize, name: Option<&V>) -> Result<()> {
        if let Some(name) = name {
            if let Some(id) = self.id_val_map.get_by_right(name) {
                self.map[index] = id.clone();
                Ok(())
            } else {
                Err(anyhow!(
                    "object with name {:?} does not exist in map {:?}",
                    name,
                    self.id_val_map
                ))
            }
        } else {
            self.map[index] = u16::MAX;
            Ok(())
        }
    }

    pub fn get(&self, loc: Vector3) -> Result<Option<&V>> {
        let index = self.index(loc)?;
        if self.map[index] == u16::MAX {
            Ok(None)
        } else {
            if let Some(name) = self.id_val_map.get_by_left(&self.map[index]) {
                Ok(Some(name))
            } else {
                Err(anyhow!(
                    "the id {} at {:?}, doesn't have a mapping in {:?}",
                    self.map[index],
                    loc,
                    self.id_val_map
                ))
            }
        }
    }

    pub fn get_id(&self, loc: Vector3) -> Result<Option<usize>> {
        let index = self.index(loc)?;
        if self.map[index] == u16::MAX {
            Ok(None)
        } else {
            Ok(Some(self.map[index] as usize))
        }
    }

    pub fn max(&self) -> u16 {
        self.max
    }
}
*/
