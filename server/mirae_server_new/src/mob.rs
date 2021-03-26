use crate::{
    gamedata::{
        gamedata::{GameData, ItemName},
        item::Item,
        mobtemplate::{InventoryBuilder, MobTemplate},
    },
    inventory::Inventory,
    stat::Stat,
    vector3::Vector3,
};
use anyhow::{anyhow, Result};
use rand::{prelude::StdRng, Rng};

const NUM_WEARS: usize = 3;

#[derive(Debug, Clone)]
pub struct Mob {
    id: u64,
    inventory: Inventory,
    drops: Inventory,
    equip: Option<ItemName>,
    wear: Inventory,
    stats: Stat,
    loc: Vector3,
}

fn get_items(
    inventory: &Inventory,
    num_items: usize,
    filter: fn(&Item) -> bool,
    g: &GameData,
    rng: &mut StdRng,
) -> Result<Vec<ItemName>> {
    let mut returned_items = Vec::new();
    let mut equippable_items = Vec::new();
    let mut equip_summed = Vec::new();
    let mut equip_sum = 0;
    for item_name in inventory.items() {
        let item = g
            .items
            .get(item_name)
            .ok_or(anyhow!(format!("invalid item name! {:?}", item_name)))?;
        if filter(item) {
            equip_sum += inventory.get(item_name);
            equip_summed.push(equip_sum);
            equippable_items.push(item_name);
        }
    }
    for _ in 0..num_items {
        let index = rng.gen_range(0, equip_sum);
        for i in 0..equip_summed.len() {
            if equip_summed[i] > index {
                returned_items.push(equippable_items[i].clone());
                break;
            }
        }
    }
    return Ok(returned_items);
}

impl Mob {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn new(
        id: u64,
        loc: Vector3,
        template: &MobTemplate,
        rng: &mut StdRng,
        g: &GameData,
    ) -> Result<Self> {
        let inventory = make_inventory(&template.tools, rng)?;
        let drops = make_inventory(&template.drops, rng)?;
        let stats = template.stats.clone();

        let mut mob = Self {
            id,
            inventory,
            equip: None,
            wear: Inventory::new(),
            drops,
            stats,
            loc,
        };

        let item = get_items(&mob.inventory, 1, |x| x.equipable, g, rng)?;
        if let Some(item_name) = item.get(0) {
            mob.equip(&item_name.clone())?;
        }

        let items = get_items(&mob.inventory, NUM_WEARS, |x| x.equipable, g, rng)?;

        Ok(mob)
    }

    pub fn dequip(&mut self) {
        if let Some(equip) = &self.equip {
            self.inventory.add(equip.clone(), 1);
            self.equip = None;
        }
    }

    pub fn unwear(&mut self, item_name: &ItemName) -> Result<()> {
        if self.wear.get(item_name) > 0 {
            self.wear.change(item_name.clone(), -1)?;
            self.inventory.add(item_name.clone(), 1);
            Ok(())
        } else {
            Err(anyhow!(format!(
                "cannot unwear {:?} because you aren't wearing it!",
                item_name
            )))
        }
    }

    pub fn unwear_all(&mut self) -> Result<()> {
        let iter: Vec<ItemName> = self.wear.items().cloned().collect();
        for item in iter {
            for _ in 0..self.wear.get(&item) {
                self.unwear(&item)?;
            }
        }
        Ok(())
    }

    pub fn equip(&mut self, item_name: &ItemName) -> Result<()> {
        self.inventory.change(item_name.clone(), -1)?;
        self.dequip();
        self.equip = Some(item_name.clone());
        Ok(())
    }

    pub fn wear(&mut self, item_name: &ItemName, g: &GameData) -> Result<()> {
        self.inventory.change(item_name.clone(), -1)?;
        if self.wear.size() < NUM_WEARS {
            let item = g
                .items
                .get(item_name)
                .ok_or(anyhow!(format!("invalid item name! {:?}", item_name)))?;
            self.stats.add_buffs(&item.buffs.stat_buffs, g);
            Ok(())
        } else {
            Err(anyhow!(format!(
                "cannot wear more than {} items!",
                NUM_WEARS
            )))
        }
    }
}

fn make_inventory(gen: &InventoryBuilder, rng: &mut StdRng) -> Result<Inventory> {
    let mut inventory = Inventory::new();
    let num_picks = rng.gen_range(gen.min, gen.max + 1);

    let mut summed = Vec::new();
    let mut sum = 0.0;
    for item in &gen.items {
        sum += item.prob;
        summed.push(sum);
    }

    for _ in 0..num_picks {
        let flt: f64 = rng.gen();
        let mut item = None;
        for i in 0..gen.items.len() {
            if flt > summed[i] {
                item = Some(&gen.items[i]);
            }
        }

        if let Some(item) = item {
            inventory.change(item.name.clone(), item.per as i64)?;
        } else {
            return Err(anyhow!("bad item generator"));
        }
    }

    Ok(inventory)
}
