use crate::{
    combat::ID,
    display::Image,
    entity::{get_items_rand, Entity, NUM_WEARS},
    gamedata::{
        gamedata::{GameData, MobName},
        item::Ability,
        mobtemplate::{InventoryBuilder, MobTemplate, Quotes},
    },
    inventory::Inventory,
    stat::Stat,
    vector3::Vector3,
};
use anyhow::Result;
use rand::{prelude::StdRng, Rng, SeedableRng};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Mob {
    id: usize,
    inventory: Inventory,
    drops: Inventory,
    equip: Inventory,
    wear: Inventory,
    stats: Stat,
    loc: Vector3,
    rng: StdRng,
    xp: i64,
    abilities: HashMap<String, Ability>,
    name: MobName,
    quotes: Quotes,
    pub display_img: String,
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
        }
    }

    Ok(inventory)
}

impl Mob {
    pub fn new(
        id: usize,
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
            equip: Inventory::new(),
            wear: Inventory::new(),
            drops,
            stats,
            loc,
            rng: SeedableRng::seed_from_u64(rng.gen()),
            xp: template.xp,
            abilities: template.abilities.clone(),
            name: template.name.clone(),
            quotes: template.quotes.clone(),
            display_img: template.display_img.clone(),
        };

        let item = get_items_rand(&mob.inventory, 1, |x| x.equipable, g, rng)?;
        if let Some(item_name) = item.get(0) {
            mob.equip(item_name, g)?;
        }

        let items = get_items_rand(&mob.inventory, NUM_WEARS, |x| x.equipable, g, rng)?;
        for item in &items {
            mob.wear(item, g)?;
        }

        Ok(mob)
    }
}

impl Entity for Mob {
    fn id(&self) -> ID {
        ID::mob(self.id)
    }

    fn inventory(&self) -> &Inventory {
        &self.inventory
    }

    fn drops(&self) -> &Inventory {
        &self.drops
    }

    fn equipped(&self) -> &Inventory {
        &self.equip
    }

    fn worn(&self) -> &Inventory {
        &self.wear
    }

    fn stats(&self) -> &Stat {
        &self.stats
    }

    fn loc(&self) -> &Vector3 {
        &self.loc
    }

    fn abilities(&self) -> HashMap<String, Ability> {
        self.abilities.clone()
    }

    fn xp(&self) -> i64 {
        self.xp
    }

    fn name(&self) -> Option<String> {
        Some(self.name.0.clone())
    }

    fn inventory_mut(&mut self) -> &mut Inventory {
        &mut self.inventory
    }

    fn drops_mut(&mut self) -> &mut Inventory {
        &mut self.drops
    }

    fn equipped_mut(&mut self) -> &mut Inventory {
        &mut self.equip
    }

    fn worn_mut(&mut self) -> &mut Inventory {
        &mut self.wear
    }

    fn stats_mut(&mut self) -> &mut Stat {
        &mut self.stats
    }

    fn loc_mut(&mut self) -> &mut Vector3 {
        &mut self.loc
    }

    fn set_xp(&mut self, xp: i64) {
        self.xp = xp;
    }

    fn rng(&mut self) -> &mut StdRng {
        &mut self.rng
    }

    fn send_display(&mut self, _: Image) { /* do nothing, mobs don't care about images */
    }
    fn send_text(&mut self, _: String) { /* do nothing, mobs don't care about text */
    }

    fn entrance(&mut self) -> Option<String> {
        let len = self.quotes.entrance.len();
        let rand = self.rng().gen_range(0, len);
        Some(self.quotes.entrance[rand].clone())
    }

    fn attack(&mut self) -> Option<String> {
        let len = self.quotes.attack.len();
        let rand = self.rng().gen_range(0, len);
        Some(self.quotes.attack[rand].clone())
    }

    fn run(&mut self) -> Option<String> {
        let len = self.quotes.run.len();
        let rand = self.rng().gen_range(0, len);
        Some(self.quotes.run[rand].clone())
    }

    fn victory(&mut self) -> Option<String> {
        let len = self.quotes.mob_victory.len();
        let rand = self.rng().gen_range(0, len);
        Some(self.quotes.mob_victory[rand].clone())
    }

    fn loss(&mut self) -> Option<String> {
        let len = self.quotes.player_victory.len();
        let rand = self.rng().gen_range(0, len);
        Some(self.quotes.player_victory[rand].clone())
    }

    fn send_image(&mut self, _: String) {}
}
