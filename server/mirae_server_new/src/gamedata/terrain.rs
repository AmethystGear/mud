use super::{
    gamedata::{BiomeName, BlockName, StructureName},
    serde_defaults::*,
};
use crate::vector3::Vector3;
use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Deserialize)]
pub struct TerrainDeser {
    pub dim: Vector3,
    pub octaves: u8,
    pub biome_octaves : u8,
    pub full_passes: Vec<TerrainPassDeser>,
    pub structure_spawn: HashMap<String, StructureSpawnDeser>,
}

#[derive(Debug)]
pub struct Terrain {
    pub dim: Vector3,
    pub octaves: u8,
    pub biome_octaves : u8,
    pub full_passes: Vec<TerrainPass>,
    pub structure_spawn: HashMap<StructureName, Vec<BiomePair>>,
}

impl TerrainDeser {
    pub fn into_terrain(
        self,
        biome_names: &HashSet<BiomeName>,
        structure_names: &HashSet<StructureName>,
    ) -> Result<Terrain> {
        let mut full_passes = Vec::new();
        for full_pass in self.full_passes {
            full_passes.push(full_pass.into_terrainpass(biome_names)?);
        }
        let biome_names_check = map_key(self.structure_spawn, structure_names)?;
        let mut structure_spawn = HashMap::new();
        for (key, val) in biome_names_check {
            structure_spawn.insert(key, val.into_structurespawn(biome_names)?);
        }
        Ok(Terrain {
            dim: self.dim,
            octaves: self.octaves,
            biome_octaves: self.biome_octaves,
            full_passes,
            structure_spawn,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct TerrainPassDeser {
    pub change_bounding_noise_per_pass: bool,
    pub layers: Vec<Vec<SinglePassDeser>>,
}

impl TerrainPassDeser {
    pub fn into_terrainpass(self, biome_names: &HashSet<BiomeName>) -> Result<TerrainPass> {
        let mut layers = Vec::new();
        for layer in self.layers {
            let mut passes = Vec::new();
            for pass in layer {
                passes.push(pass.into_singlepass(biome_names)?);
            }
            layers.push(passes);
        }

        Ok(TerrainPass {
            change_bounding_noise_per_pass: self.change_bounding_noise_per_pass,
            layers,
        })
    }
}

#[derive(Debug)]
pub struct TerrainPass {
    pub change_bounding_noise_per_pass: bool,
    pub layers: Vec<Vec<SinglePass>>,
}

#[derive(Debug, Deserialize)]
pub struct SinglePassDeser {
    pub biome: String,
    #[serde(default = "f64_two")]
    pub cutoff: f64,
}

impl SinglePassDeser {
    pub fn into_singlepass(self, biome_names: &HashSet<BiomeName>) -> Result<SinglePass> {
        let name = self.biome.into();
        if biome_names.contains(&name) {
            Ok(SinglePass {
                biome: name,
                cutoff: self.cutoff,
            })
        } else {
            Err(anyhow!(format!("there is no biome with name {:?}", name)))
        }
    }
}

#[derive(Debug)]
pub struct SinglePass {
    pub biome: BiomeName,
    pub cutoff: f64,
}

fn f64_two() -> f64 {
    2.0
}

fn f64_neg_one() -> f64 {
    -1.0
}

#[derive(Debug, Deserialize)]
pub struct BiomePairDeser {
    pub biome: String,
    #[serde(default = "f64_neg_one")]
    pub prob: f64,
}

#[derive(Debug)]
pub struct BiomePair {
    pub biome: BiomeName,
    pub prob: f64,
}

impl BiomePairDeser {
    pub fn into_biomepair(
        self,
        biome_names: &HashSet<BiomeName>,
        default_prob: f64,
    ) -> Result<BiomePair> {
        let name = self.biome.into();
        if biome_names.contains(&name) {
            Ok(BiomePair {
                biome: name,
                prob: if self.prob < 0.0 {
                    default_prob
                } else {
                    self.prob
                },
            })
        } else {
            Err(anyhow!(format!("there is no biome with name {:?}", name)))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct StructureSpawnDeser {
    pub biomes: Vec<BiomePairDeser>,
    #[serde(default = "f64_neg_one")]
    pub default_prob: f64,
}

impl StructureSpawnDeser {
    pub fn into_structurespawn(self, biome_names: &HashSet<BiomeName>) -> Result<Vec<BiomePair>> {
        let mut res = Vec::new();
        for biome in self.biomes {
            res.push(biome.into_biomepair(biome_names, self.default_prob)?);
        }
        Ok(res)
    }
}

#[derive(Debug)]
pub struct BlockCutoff {
    pub then: BlockName,
    pub above: f64,
}
#[derive(Debug, Deserialize)]
pub struct BlockCutoffDeser {
    pub then: String,
    #[serde(default = "f64_neg_one")]
    pub above: f64,
}

impl BlockCutoffDeser {
    pub fn into_blockcutoff(self, block_names: &HashSet<BlockName>) -> Result<BlockCutoff> {
        let name = self.then.into();
        if block_names.contains(&name) {
            Ok(BlockCutoff {
                then: name,
                above: self.above,
            })
        } else {
            Err(anyhow!(format!("there is no block with name {:?}", name)))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BiomeDeser {
    #[serde(default = "empty_vec")]
    pub terrain_pass: Vec<BlockCutoffDeser>,
    #[serde(default = "empty_vec")]
    pub biome_pass: Vec<BlockCutoffDeser>,
}

#[derive(Debug)]
pub struct Biome {
    pub name: BiomeName,
    pub terrain_pass: Vec<BlockCutoff>,
    pub biome_pass: Vec<BlockCutoff>,
}

impl BiomeDeser {
    pub fn into_biome(
        self,
        name: BiomeName,
        structure_names: &HashSet<StructureName>,
        block_names: &HashSet<BlockName>,
    ) -> Result<Biome> {
        let mut terrain_pass = Vec::new();
        for block_cutoff in self.terrain_pass {
            terrain_pass.push(block_cutoff.into_blockcutoff(block_names)?);
        }

        let mut biome_pass = Vec::new();
        for block_cutoff in self.biome_pass {
            biome_pass.push(block_cutoff.into_blockcutoff(block_names)?);
        }
        Ok(Biome {
            name,
            terrain_pass,
            biome_pass,
        })
    }
}
