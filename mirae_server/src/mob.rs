use crate::gamedata::item::Item;
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
use crate::{
    gamedata::gamedata::{DmgType, ItemName},
    stat::default_empty_fields,
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
    attack_buffs: HashMap<DmgType, f64>,
    defense_buffs: HashMap<DmgType, f64>,
    pub display_img: String,
}

fn make_inventory(gen: &InventoryBuilder, rng: &mut StdRng, g: &GameData) -> Result<Inventory> {
    let mut inventory = Inventory::new();
    let num_picks = rng.gen_range(gen.min, gen.max + 1);
    if gen.tags.len() != 0 {
        let all_items: Vec<ItemName> = g.items.keys().cloned().collect();

        let add_if_tags_match =
            |inventory: &mut Inventory, rand_item: &Item, rand_item_name: &ItemName| {
                for tag0 in &rand_item.tags {
                    for tag1 in &gen.tags {
                        if tag0 == tag1 {
                            inventory.add(rand_item_name.clone(), 1);
                            return;
                        }
                    }
                }
            };
        while inventory.size() < num_picks {
            let rand_item_name = &all_items[rng.gen_range(0, all_items.len())];
            let rand_item = &g.items[rand_item_name];
            add_if_tags_match(&mut inventory, rand_item, rand_item_name);
        }
    } else {
        let mut summed = Vec::new();
        let mut sum = 0.0;
        for prob in &gen.probs {
            sum += prob;
            summed.push(sum);
        }

        for _ in 0..num_picks {
            let flt: f64 = rng.gen();
            let mut items = None;
            for i in 0..gen.items.len() {
                if flt > summed[i] {
                    items = Some(&gen.items[i]);
                }
            }
            if let Some(items) = items {
                for item in items {
                    inventory.add(item.name.clone(), item.per);
                }
            }
        }
    }
    for item in &gen.required_items {
        inventory.add(item.name.clone(), item.per);
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
        let inventory = make_inventory(&template.tools, rng, g)?;
        let drops = make_inventory(&template.drops, rng, g)?;
        let stats = template.stats.clone();

        let buffs = default_empty_fields(&HashMap::new(), 1.0, &g.dmg);
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
            attack_buffs: buffs.clone(),
            defense_buffs: buffs,
        };

        let item = get_items_rand(mob.inventory(), 1, |x| x.equipable, g, rng)?;
        if let Some(item_name) = item.get(0) {
            mob.equip(item_name, g)?;
        }

        let items = get_items_rand(mob.inventory(), NUM_WEARS, |x| x.wearable, g, rng)?;
        for item_name in &items {
            mob.wear(item_name, g)?;
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

    fn name(&self) -> String {
        self.name.0.clone()
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

    fn send_display(&mut self, _: Image, _: bool) { /* do nothing, mobs don't care about images */
    }
    fn send_text(&mut self, _: String) { /* do nothing, mobs don't care about text */
    }

    fn entrance(&mut self) -> Result<String> {
        let len = self.quotes.entrance.len();
        let rand = self.rng().gen_range(0, len);
        Ok(self.quotes.entrance[rand].clone())
    }

    fn attack(&mut self) -> Result<String> {
        let len = self.quotes.attack.len();
        let rand = self.rng().gen_range(0, len);
        Ok(self.quotes.attack[rand].clone())
    }

    fn run(&mut self) -> Result<String> {
        let len = self.quotes.run.len();
        let rand = self.rng().gen_range(0, len);
        Ok(self.quotes.run[rand].clone())
    }

    fn victory(&mut self) -> Result<String> {
        let len = self.quotes.mob_victory.len();
        let rand = self.rng().gen_range(0, len);
        Ok(self.quotes.mob_victory[rand].clone())
    }

    fn loss(&mut self) -> Result<String> {
        let len = self.quotes.player_victory.len();
        let rand = self.rng().gen_range(0, len);
        Ok(self.quotes.player_victory[rand].clone())
    }

    fn send_image(&mut self, _: String) {}

    fn attack_buffs(&self) -> &HashMap<DmgType, f64> {
        &self.attack_buffs
    }

    fn attack_buffs_mut(&mut self) -> &mut HashMap<DmgType, f64> {
        &mut self.attack_buffs
    }

    fn defense_buffs(&self) -> &HashMap<DmgType, f64> {
        &self.defense_buffs
    }

    fn defense_buffs_mut(&mut self) -> &mut HashMap<DmgType, f64> {
        &mut self.defense_buffs
    }
}
