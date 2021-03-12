use crate::{
    gamedata::{BiomeName, BlockName, StructureName},
    vector3::Vector3,
};
use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
pub struct TerrainDeser {
    pub dim: Vector3,
    pub octaves: u8,
    pub full_passes: Vec<TerrainPassDeser>,
}

#[derive(Debug)]
pub struct Terrain {
    pub dim: Vector3,
    pub octaves: u8,
    pub full_passes: Vec<TerrainPass>,
}

impl TerrainDeser {
    pub fn into_terrain(self, biome_names: &HashSet<BiomeName>) -> Result<Terrain> {
        let mut full_passes = Vec::new();
        for full_pass in self.full_passes {
            full_passes.push(full_pass.into_terrainpass(biome_names)?);
        }
        Ok(Terrain {
            dim: self.dim,
            octaves: self.octaves,
            full_passes,
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
    #[serde(default = "default_cutoff")]
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

fn default_cutoff() -> f64 {
    2.0
}
fn default_above() -> f64 {
    -1.0
}

#[derive(Debug)]
pub struct StructureSpawn {
    pub structure: StructureName,
    pub prob: f64,
}

#[derive(Debug, Deserialize)]
pub struct StructureSpawnDeser {
    pub structure: String,
    pub prob: f64,
}

impl StructureSpawnDeser {
    pub fn into_structurespawn(
        self,
        block_names: &HashSet<StructureName>,
    ) -> Result<StructureSpawn> {
        let name = self.structure.into();
        if block_names.contains(&name) {
            Ok(StructureSpawn {
                structure: name,
                prob: self.prob,
            })
        } else {
            Err(anyhow!(format!(
                "there is no structure with name {:?}",
                name
            )))
        }
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
    #[serde(default = "default_above")]
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
    pub terrain_pass: Vec<BlockCutoffDeser>,
    pub biome_pass: Vec<BlockCutoffDeser>,
    pub spawn: Vec<StructureSpawnDeser>,
}

#[derive(Debug)]
pub struct Biome {
    pub name: BiomeName,
    pub terrain_pass: Vec<BlockCutoff>,
    pub biome_pass: Vec<BlockCutoff>,
    pub spawn: Vec<StructureSpawn>,
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

        let mut spawn = Vec::new();
        for structure_spawn in self.spawn {
            spawn.push(structure_spawn.into_structurespawn(structure_names)?);
        }

        Ok(Biome {
            name,
            terrain_pass,
            biome_pass,
            spawn,
        })
    }
}
