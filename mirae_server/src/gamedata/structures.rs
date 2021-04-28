use super::{
    gamedata::{BlockName, GameData, MobName, StructureName},
    serde_defaults::empty_vec,
};
use crate::{
    rgb::RGB,
    vector3::Vector3,
    world::{Map, MobU16},
};
use anyhow::{anyhow, Result};
use image::io::Reader as ImageReader;
use rand::{prelude::StdRng, Rng};
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::Arc,
};

#[derive(Deserialize, Debug)]
struct MobAndProbDeser {
    mob: String,
    prob: f64,
}

#[derive(Deserialize, Debug)]
struct StructureMappingDeser {
    color: RGB,
    block: String,
    #[serde(default = "empty_vec")]
    mobs: Vec<MobAndProbDeser>,
}

#[derive(Deserialize, Debug)]
pub struct StructureDeser {
    sources: Vec<String>,
    mapping: Vec<StructureMappingDeser>,
}

impl StructureDeser {
    pub fn get_structures(
        self,
        folder: &Path,
        block_names: &HashSet<BlockName>,
        mob_names: &HashSet<MobName>,
        structure_name: StructureName,
    ) -> Result<Vec<Structure>> {
        let mut v = Vec::new();

        let mut i = 0;
        
        let mut rgb_to_block = HashMap::new();
        let mut mob_and_prob = Vec::new();
        for sm in self.mapping {
            let mut vec = Vec::new();
            let mut sum = 0.0;
            for v in sm.mobs {
                let mob_name = MobName::from(v.mob);
                if !mob_names.contains(&mob_name) {
                    return Err(anyhow!(format!("{:?} isn't a mob", mob_name)));
                } else {
                    sum += v.prob;
                    vec.push(MobAndProb {
                        mob: mob_name,
                        prob: sum,
                    });
                }
            }
            mob_and_prob.push(vec);
            let name = BlockName::from(sm.block);
            if !block_names.contains(&name) {
                return Err(anyhow!(format!("{:?} isn't a block", name)));
            } else {
                rgb_to_block.insert(sm.color, (name, i));
            }
            i += 1;
        }
        
        let mob_and_prob = Arc::new(mob_and_prob);

        for source in self.sources {
            let path = folder.join(source);
            let image = ImageReader::open(path)?.decode()?;
            let image = image.as_rgba8().ok_or(anyhow!("bad image format!"))?;
            let x = image.width() as usize;
            let y = image.height() as usize;
            let dim = Vector3::new(x as isize, y as isize, 1);
            let mut block_map = Map::new(dim, None);
            let mut mob_map = Map::new(dim, None);
            let mut index = 0;
            for pix in image.pixels() {
                let alpha = pix.0[3];
                // transparent pixels are ignored
                if alpha == u8::MAX {
                    let rgb = RGB::new(pix.0[0], pix.0[1], pix.0[2]);
                    if let Some((block, mp)) = rgb_to_block.get(&rgb) {
                        block_map.direct_set(index, Some(block.clone()));
                        mob_map.direct_set(index, Some(mp.clone()));
                    } else {
                        return Err(anyhow!(format!("invalid pixel color {:?}", rgb)));
                    }
                }
                index += 1;
            }
            v.push(Structure {
                structure_name: structure_name.clone(),
                mobgen: mob_and_prob.clone(),
                blocks: block_map,
                mobs: mob_map,
            });
        }
        Ok(v)
    }
}

struct MobAndProb {
    mob: MobName,
    prob: f64,
}

pub struct Structure {
    structure_name: StructureName,
    mobgen: Arc<Vec<Vec<MobAndProb>>>,
    blocks: Map<Option<BlockName>>,
    mobs: Map<Option<usize>>,
}

impl Structure {
    pub fn spawn_at(
        &self,
        loc: Vector3,
        block_map: &mut Map<u8>,
        mob_map: &mut Map<MobU16>,
        g: &GameData,
        rng: &mut StdRng,
    ) -> Result<()> {
        for y in 0..self.blocks.dim.y() {
            for x in 0..self.blocks.dim.x() {
                let struct_posn = Vector3::new(x, y, 0);
                let posn = loc + struct_posn;
                if let Some(block) = &self.blocks.get(struct_posn)? {
                    block_map.set(posn, g.get_block_id_by_blockname(&block)?)?;
                    let block = g.blocks.name_to_item.get(block).expect("validated");
                    if block.z_passable {
                        let below = loc + Vector3::new(0, 0, 1);
                        let below_block = g.get_block_name_by_id(block_map.get(below)?)?;
                        let below_block = g.blocks.name_to_item.get(&below_block).expect("validated");
                        if below_block.solid {
                            block_map.set(below, g.get_block_id_by_name("stone")?)?;
                        }
                    }
                }
                if let Some(mob_index) = &self.mobs.get(struct_posn)? {
                    let mobs = &self.mobgen[mob_index.clone()];
                    let chance: f64 = rng.gen();
                    for m in mobs {
                        if chance < m.prob {
                            mob_map.set(posn, g.get_mob_id_by_name(&m.mob)?)?;
                            break;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
