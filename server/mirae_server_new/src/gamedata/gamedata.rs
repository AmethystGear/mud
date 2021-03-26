use super::{
    block::{Block, BlockDeser},
    item::{Item, ItemDeser},
    mobtemplate::{MobTemplate, MobTemplateDeser},
    structures::{Structure, StructureDeser},
    terrain::{Biome, BiomeDeser, Terrain, TerrainDeser},
};
use crate::{
    playerout::{Packet, PacketType},
    world::MobU16,
};
use anyhow::{anyhow, Result};
use bimap::BiMap;
use serde::Deserialize;
use serde_jacl::de::from_str;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DmgType(String);

impl From<String> for DmgType {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl DmgType {
    fn checked_from(s: String, g: &GameData) -> Result<Self> {
        let val = Self(s);
        if g.dmg.contains(&val) {
            Ok(val)
        } else {
            Err(anyhow!(format!("{:?} not in {:?}", val, g.dmg)))
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StatType(String);

impl From<String> for StatType {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl StatType {
    pub fn checked_from(s: String, g: &GameData) -> Result<Self> {
        let val = Self(s);
        if g.stat.contains(&val) {
            Ok(val)
        } else {
            Err(anyhow!(format!("{:?} not in {:?}", val, g.stat)))
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StructureName(String);

impl From<String> for StructureName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl StructureName {
    fn checked_from(s: String, g: &GameData) -> Result<Self> {
        let val = Self(s);
        if g.structures.contains_key(&val) {
            Ok(val)
        } else {
            Err(anyhow!(format!(
                "{:?} not in {:?}",
                val,
                g.structures.keys().collect::<Vec<&StructureName>>()
            )))
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct BiomeName(String);

impl From<String> for BiomeName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl BiomeName {
    fn checked_from(s: String, g: &GameData) -> Result<Self> {
        let val = Self(s);
        if g.biomes.contains_key(&val) {
            Ok(val)
        } else {
            Err(anyhow!(format!("{:?} not in {:?}", val, g.biomes)))
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ItemName(String);

impl From<String> for ItemName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl ItemName {
    fn checked_from(s: String, g: &GameData) -> Result<Self> {
        let val = Self(s);
        if g.items.contains_key(&val) {
            Ok(val)
        } else {
            Err(anyhow!(format!("{:?} not in {:?}", val, g.items.keys())))
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct MobName(String);

impl From<String> for MobName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl MobName {
    fn checked_from(s: String, g: &GameData) -> Result<Self> {
        let val = Self(s);
        if g.mob_templates.contains_key(&val) {
            Ok(val)
        } else {
            Err(anyhow!(format!(
                "{:?} not in {:?}",
                val,
                g.mob_templates.keys()
            )))
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct BlockName(String);

impl From<String> for BlockName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl BlockName {
    fn checked_from(s: String, g: &GameData) -> Result<Self> {
        let val = Self(s);
        if g.blocks.contains_key(&val) {
            Ok(val)
        } else {
            Err(anyhow!(format!("{:?} not in {:?}", val, g.blocks.keys())))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GameMode {
    items: String,
    terrain: String,
    biomes: String,
    structures: String,
    dmg: String,
    stat: String,
    mobs: String,
    blocks: String,
}

impl GameMode {
    pub fn into_gamedata(&self) -> Result<GameData> {
        let deser = GameDataDeser {
            terrain: from_str(&fs::read_to_string(&self.terrain)?)?,
            dmg: from_str(&fs::read_to_string(&self.dmg)?)?,
            stat: from_str(&fs::read_to_string(&self.stat)?)?,
            items: from_str(&fs::read_to_string(&self.items)?)?,
            mob_templates: from_str(&fs::read_to_string(&self.mobs)?)?,
            blocks: from_str(&fs::read_to_string(&self.blocks)?)?,
            structures: from_str(&fs::read_to_string(&self.structures)?)?,
            biomes: from_str(&fs::read_to_string(&self.biomes)?)?,
        };

        let dmg_types = deser.dmg.into_iter().map(|x| DmgType(x)).collect();
        let stat_types = deser.stat.into_iter().map(|x| StatType(x)).collect();
        let item_names = deser.items.keys().map(|x| ItemName(x.clone())).collect();

        let mut items = HashMap::new();
        for (name, v) in deser.items {
            let name = ItemName::from(name);
            items.insert(
                name.clone(),
                v.into_item(&dmg_types, &stat_types, &item_names, name)?,
            );
        }

        let mut mob_names = HashSet::new();
        let mut mob_templates = HashMap::new();
        for (name, v) in deser.mob_templates {
            let name = MobName::from(name);
            mob_names.insert(name.clone());
            mob_templates.insert(
                name.clone(),
                v.into_mobtemplate(&dmg_types, &item_names, &stat_types, name)?,
            );
        }

        let block_names = deser
            .blocks
            .keys()
            .into_iter()
            .cloned()
            .map(|x| BlockName(x))
            .collect();

        let blocks: HashMap<BlockName, Block> = deser
            .blocks
            .into_iter()
            .map(|(name, block)| {
                (
                    BlockName::from(name.clone()),
                    block.into_block(BlockName::from(name)),
                )
            })
            .collect();

        let folder = Path::new(&self.structures)
            .parent()
            .ok_or(anyhow!("invalid path"))?;
        let mut structure_names = HashSet::new();
        let mut structures = HashMap::new();
        for (name, v) in deser.structures {
            let name = StructureName(name);
            structure_names.insert(name.clone());
            structures.insert(
                name.clone(),
                v.get_structures(folder, &block_names, &mob_names, name)?,
            );
        }

        let mut biomes = HashMap::new();
        let mut biome_names = HashSet::new();
        for (name, v) in deser.biomes {
            let name = BiomeName(name);
            biome_names.insert(name.clone());
            biomes.insert(
                name.clone(),
                v.into_biome(name, &structure_names, &block_names)?,
            );
        }

        let terrain = deser.terrain.into_terrain(&biome_names)?;

        GameData::new(
            terrain,
            dmg_types,
            stat_types,
            items,
            mob_templates,
            blocks,
            biomes,
            structures,
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct GameDataDeser {
    terrain: TerrainDeser,
    dmg: Vec<String>,
    stat: Vec<String>,
    items: HashMap<String, ItemDeser>,
    mob_templates: HashMap<String, MobTemplateDeser>,
    blocks: HashMap<String, BlockDeser>,
    biomes: HashMap<String, BiomeDeser>,
    structures: HashMap<String, StructureDeser>,
}

pub struct GameData {
    pub terrain: Terrain,
    pub dmg: HashSet<DmgType>,
    pub stat: HashSet<StatType>,
    pub biomes: HashMap<BiomeName, Biome>,
    pub structures: HashMap<StructureName, Vec<Structure>>,
    pub items: HashMap<ItemName, Item>,
    pub mob_templates: HashMap<MobName, MobTemplate>,
    pub mob_id_map: BiMap<MobU16, MobName>,
    pub max_mob_id: MobU16,
    pub blocks: HashMap<BlockName, Block>,
    pub block_id_map: BiMap<u8, BlockName>,
    pub max_block_id: u8,
    pub init_packet: Packet,
    pub mob_id_to_img_id: HashMap<u16, u8>,
}

impl GameData {
    pub fn new(
        terrain: Terrain,
        dmg: HashSet<DmgType>,
        stat: HashSet<StatType>,
        items: HashMap<ItemName, Item>,
        mob_templates: HashMap<MobName, MobTemplate>,
        blocks: HashMap<BlockName, Block>,
        biomes: HashMap<BiomeName, Biome>,
        structures: HashMap<StructureName, Vec<Structure>>,
    ) -> Result<Self> {
        let mut mob_id_map: BiMap<MobU16, MobName> = BiMap::new();
        let mut max_mob_id = MobU16(0);
        for name in mob_templates.keys() {
            if max_mob_id == MobU16::empty() {
                return Err(anyhow!(format!(
                    "number of mobs cannot exceed {:?}",
                    MobU16::empty()
                )));
            }
            mob_id_map.insert(max_mob_id, name.clone());
            max_mob_id = MobU16(max_mob_id.0 + 1);
        }

        let mut block_id_map: BiMap<u8, BlockName> = BiMap::new();
        let mut max_block_id = 0;
        for name in blocks.keys() {
            if max_block_id == u8::MAX {
                return Err(anyhow!(format!(
                    "number of blocks cannot exceed {}",
                    u8::MAX
                )));
            }
            block_id_map.insert(max_block_id, name.clone());
            max_block_id += 1
        }

        let mut img_to_img_id = HashMap::new();
        let mut mob_id_to_img_id = HashMap::new();
        let mut id: u8 = 0;
        for i in 0..max_mob_id.0 {
            let mob_name = mob_id_map
                .get_by_left(&MobU16(i))
                .ok_or(anyhow!("invalid id"))?;
            let mob_template = mob_templates
                .get(&mob_name)
                .ok_or(anyhow!("invalid mob name"))?;

            if !img_to_img_id.contains_key(&mob_template.display) {
                img_to_img_id.insert(mob_template.display.clone(), id);
                id += 1;
            }
            let val = img_to_img_id
                .get(&mob_template.display)
                .expect("this should never happen");

            mob_id_to_img_id.insert(i, val.clone());
        }
        let mut img_id_to_img = HashMap::new();
        for (k, v) in img_to_img_id {
            img_id_to_img.insert(format!("{}", v), k);
        }

        Ok(GameData {
            terrain,
            dmg,
            stat,
            items,
            mob_templates,
            mob_id_map,
            blocks,
            block_id_map,
            biomes,
            max_block_id,
            max_mob_id,
            structures,
            init_packet: Packet {
                p_type: PacketType::Init,
                content: serde_json::to_string(&img_id_to_img)?.into_bytes(),
            },
            mob_id_to_img_id,
        })
    }

    pub fn get_mob_name_by_id(&self, id: MobU16) -> Result<MobName> {
        Ok(self
            .mob_id_map
            .get_by_left(&id)
            .ok_or_else(|| anyhow!(format!("invalid id {:?}", id)))?
            .clone())
    }

    pub fn get_mob_id_by_name(&self, name: &MobName) -> Result<MobU16> {
        Ok(self
            .mob_id_map
            .get_by_right(name)
            .ok_or_else(|| anyhow!(format!("invalid name {:?}", name)))?
            .clone())
    }

    pub fn get_block_name_by_id(&self, id: u8) -> Result<BlockName> {
        Ok(self
            .block_id_map
            .get_by_left(&id)
            .ok_or_else(|| anyhow!(format!("invalid id {}", id)))?
            .clone())
    }

    pub fn get_block_id_by_name<S: Into<String>>(&self, name: S) -> Result<u8> {
        let name = BlockName::checked_from(name.into(), self)?;
        self.get_block_id_by_blockname(&name)
    }

    pub fn get_block_id_by_blockname(&self, name: &BlockName) -> Result<u8> {
        Ok(self
            .block_id_map
            .get_by_right(name)
            .ok_or_else(|| anyhow!(format!("invalid name {:?}", name)))?
            .clone())
    }
}
