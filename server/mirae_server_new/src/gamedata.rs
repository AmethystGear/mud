use crate::{
    block::{Block, BlockDeser},
    item::{Item, ItemDeser},
    mob::{MobTemplate, MobTemplateDeser},
    terrain::{Biome, BiomeDeser, Terrain, TerrainDeser},
};
use anyhow::{anyhow, Result};
use bimap::BiMap;
use serde::Deserialize;
use serde_jacl::de::from_str;
use std::{
    collections::{HashMap, HashSet},
    fs,
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
    fn checked_from(s: String, g: &GameData) -> Result<Self> {
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
        if g.structures.contains(&val) {
            Ok(val)
        } else {
            Err(anyhow!(format!("{:?} not in {:?}", val, g.structures)))
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
pub struct MobAction(String);

impl From<String> for MobAction {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl MobAction {
    fn checked_from(s: String, g: &GameData) -> Result<Self> {
        let val = Self(s);
        if g.mob_actions.contains(&val) {
            Ok(val)
        } else {
            Err(anyhow!(format!("{:?} not in {:?}", val, g.mob_actions)))
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
    mob_actions: String,
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
            mob_actions: from_str(&fs::read_to_string(&self.mob_actions)?)?,
            mob_templates: from_str(&fs::read_to_string(&self.mobs)?)?,
            blocks: from_str(&fs::read_to_string(&self.blocks)?)?,
            biomes: from_str(&fs::read_to_string(&self.biomes)?)?,
            structures: from_str(&fs::read_to_string(&self.structures)?)?,
        };
        let structures = deser
            .structures
            .into_iter()
            .map(|x| StructureName(x))
            .collect();
        let dmg_types = deser.dmg.into_iter().map(|x| DmgType(x)).collect();
        let stat_types = deser.stat.into_iter().map(|x| StatType(x)).collect();
        let mob_actions = deser
            .mob_actions
            .into_iter()
            .map(|x| MobAction(x))
            .collect();
        let item_names = deser.items.keys().map(|x| ItemName(x.clone())).collect();

        let mut items = HashMap::new();
        for (name, v) in deser.items {
            let name = ItemName::from(name);
            items.insert(
                name.clone(),
                v.into_item(&dmg_types, &stat_types, &item_names, name)?,
            );
        }

        let mut mob_templates = HashMap::new();
        for (name, v) in deser.mob_templates {
            let name = MobName::from(name);
            mob_templates.insert(
                name.clone(),
                v.into_mobtemplate(&dmg_types, &item_names, &mob_actions, name)?,
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

        let mut biomes = HashMap::new();
        let mut biome_names = HashSet::new();
        for (name, v) in deser.biomes {
            let name = BiomeName(name);
            biome_names.insert(name.clone());
            biomes.insert(name.clone(), v.into_biome(name, &structures, &block_names)?);
        }

        let terrain = deser.terrain.into_terrain(&biome_names)?;

        GameData::new(
            terrain,
            dmg_types,
            stat_types,
            items,
            mob_templates,
            mob_actions,
            blocks,
            structures,
            biomes,
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct GameDataDeser {
    terrain: TerrainDeser,
    structures: Vec<String>,
    dmg: Vec<String>,
    stat: Vec<String>,
    items: HashMap<String, ItemDeser>,
    mob_actions: Vec<String>,
    mob_templates: HashMap<String, MobTemplateDeser>,
    blocks: HashMap<String, BlockDeser>,
    biomes: HashMap<String, BiomeDeser>,
}

pub struct GameData {
    pub terrain: Terrain,
    pub dmg: HashSet<DmgType>,
    pub stat: HashSet<StatType>,
    pub structures: HashSet<StructureName>,
    pub biomes: HashMap<BiomeName, Biome>,
    pub items: HashMap<ItemName, Item>,
    pub mob_templates: HashMap<MobName, MobTemplate>,
    pub mob_actions: HashSet<MobAction>,
    pub mob_id_map: BiMap<u16, MobName>,
    pub blocks: HashMap<BlockName, Block>,
    pub block_id_map: BiMap<u8, BlockName>,
}

impl GameData {
    pub fn new(
        terrain: Terrain,
        dmg: HashSet<DmgType>,
        stat: HashSet<StatType>,
        items: HashMap<ItemName, Item>,
        mob_templates: HashMap<MobName, MobTemplate>,
        mob_actions: HashSet<MobAction>,
        blocks: HashMap<BlockName, Block>,
        structures: HashSet<StructureName>,
        biomes: HashMap<BiomeName, Biome>,
    ) -> Result<Self> {
        let mut mob_id_map: BiMap<u16, MobName> = BiMap::new();
        let mut id = 0;
        for name in mob_templates.keys() {
            if id == u16::MAX {
                return Err(anyhow!(format!(
                    "number of mobs cannot exceed {}",
                    u16::MAX
                )));
            }
            mob_id_map.insert(id, name.clone());
            id += 1
        }

        let mut block_id_map: BiMap<u8, BlockName> = BiMap::new();
        let mut id = 0;
        for name in blocks.keys() {
            block_id_map.insert(id, name.clone());
            id += 1
        }

        Ok(GameData {
            terrain,
            dmg,
            stat,
            items,
            mob_templates,
            mob_actions,
            mob_id_map,
            blocks,
            block_id_map,
            structures,
            biomes,
        })
    }

    pub fn get_mob_name_by_id(&self, id: u16) -> Result<MobName> {
        Ok(self
            .mob_id_map
            .get_by_left(&id)
            .ok_or_else(|| anyhow!(format!("invalid id {}", id)))?
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
