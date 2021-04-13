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
use serde::{Serialize, Deserialize};
use serde_jacl::de::from_str;
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    fs,
    hash::Hash,
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
    pub fn checked_from(s: String, g: &GameData) -> Result<Self> {
        let val = Self(s);
        if g.dmg.contains(&val) {
            Ok(val)
        } else {
            Err(anyhow!(format!("{} is not a damage type", val.0)))
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
            Err(anyhow!(format!("{} is not a stat", val.0)))
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct StructureName(String);

impl From<String> for StructureName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl StructureName {
    pub fn checked_from(s: String, g: &GameData) -> Result<Self> {
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
    pub fn checked_from(s: String, g: &GameData) -> Result<Self> {
        let val = Self(s);
        if g.biomes.name_to_item.contains_key(&val) {
            Ok(val)
        } else {
            Err(anyhow!(format!(
                "{:?} not in {:?}",
                val,
                g.biomes.name_to_item.keys()
            )))
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ItemName(pub String);

impl From<String> for ItemName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl ItemName {
    pub fn checked_from(s: String, g: &GameData) -> Result<Self> {
        let val = Self(s);
        if g.items.contains_key(&val) {
            Ok(val)
        } else {
            Err(anyhow!(format!("{} is not an item", val.0)))
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct MobName(pub String);

impl From<String> for MobName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl MobName {
    pub fn checked_from(s: String, g: &GameData) -> Result<Self> {
        let val = Self(s);
        if g.mob_templates.name_to_item.contains_key(&val) {
            Ok(val)
        } else {
            Err(anyhow!(format!(
                "{:?} not in {:?}",
                val,
                g.mob_templates.name_to_item.keys()
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
    pub fn checked_from(s: String, g: &GameData) -> Result<Self> {
        let val = Self(s);
        if g.blocks.name_to_item.contains_key(&val) {
            Ok(val)
        } else {
            Err(anyhow!(format!(
                "{:?} not in {:?}",
                val,
                g.blocks.name_to_item.keys()
            )))
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

        let terrain = deser.terrain.into_terrain(&biome_names, &structure_names)?;

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

pub struct IDMap<A, B, C> {
    pub name_to_item: HashMap<B, C>,
    pub id_to_name: BiMap<A, B>,
    pub max_id: A,
}

pub struct GameData {
    pub terrain: Terrain,
    pub dmg: HashSet<DmgType>,
    pub stat: HashSet<StatType>,
    pub structures: HashMap<StructureName, Vec<Structure>>,
    pub items: HashMap<ItemName, Item>,
    pub biomes: IDMap<u8, BiomeName, Biome>,
    pub mob_templates: IDMap<MobU16, MobName, MobTemplate>,
    pub blocks: IDMap<u8, BlockName, Block>,
    pub init_packet: Packet,
    pub mob_id_to_img_id: HashMap<u16, u8>,
}

fn get_idmap<A: Eq + Debug + Hash + Copy, B: Hash + Eq + Clone, C>(
    map: HashMap<B, C>,
    increment: fn(A) -> A,
    start: A,
    max: A,
) -> Result<IDMap<A, B, C>> {
    let mut id_map = BiMap::new();
    let mut max_id = start;
    for name in map.keys() {
        if max_id == max {
            return Err(anyhow!(format!("number of mobs cannot exceed {:?}", max)));
        }
        id_map.insert(max_id, name.clone());
        max_id = increment(max_id);
    }
    Ok(IDMap {
        name_to_item: map,
        id_to_name: id_map,
        max_id,
    })
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
        let mut img_to_img_id = HashMap::new();
        let mut mob_id_to_img_id = HashMap::new();
        let mut id: u8 = 0;
        let mob_templates = get_idmap(
            mob_templates,
            |x| MobU16(x.0 + 1),
            MobU16(0),
            MobU16::empty(),
        )?;

        let mut images_to_load = HashSet::new();
        for i in 0..mob_templates.max_id.0 {
            let mob_name = mob_templates
                .id_to_name
                .get_by_left(&MobU16(i))
                .ok_or(anyhow!("invalid id"))?;
            let mob_template = mob_templates
                .name_to_item
                .get(&mob_name)
                .ok_or(anyhow!("invalid mob name"))?;

            images_to_load.insert(mob_template.display_img.clone());

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

        let images_to_load: Vec<String> = images_to_load.into_iter().collect();

        #[derive(Serialize)]
        struct Content {
            img_id_to_img: HashMap<String, String>,
            images_to_load: Vec<String>,
        }

        Ok(GameData {
            terrain,
            dmg,
            stat,
            items,
            blocks: get_idmap(blocks, |x| x + 1, 0u8, u8::MAX)?,
            mob_templates,
            biomes: get_idmap(biomes, |x| x + 1, 0u8, u8::MAX)?,
            structures,
            init_packet: Packet {
                p_type: PacketType::Init,
                content: serde_json::to_string(&Content {
                    img_id_to_img,
                    images_to_load,
                })?
                .into_bytes(),
            },
            mob_id_to_img_id,
        })
    }

    pub fn get_mob_name_by_id(&self, id: MobU16) -> Result<MobName> {
        Ok(self
            .mob_templates
            .id_to_name
            .get_by_left(&id)
            .ok_or_else(|| anyhow!(format!("invalid id {:?}", id)))?
            .clone())
    }

    pub fn get_mob_id_by_name(&self, name: &MobName) -> Result<MobU16> {
        Ok(self
            .mob_templates
            .id_to_name
            .get_by_right(name)
            .ok_or_else(|| anyhow!(format!("invalid name {:?}", name)))?
            .clone())
    }

    pub fn get_block_name_by_id(&self, id: u8) -> Result<BlockName> {
        Ok(self
            .blocks
            .id_to_name
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
            .blocks
            .id_to_name
            .get_by_right(name)
            .ok_or_else(|| anyhow!(format!("invalid name {:?}", name)))?
            .clone())
    }
}