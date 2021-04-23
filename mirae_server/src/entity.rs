use crate::{
    combat::{BattleMap, StatusEffect, ID},
    display::Image,
    gamedata::{
        gamedata::{DmgType, GameData, ItemName},
        item::{Ability, Item},
    },
    inventory::Inventory,
    stat::Stat,
    vector3::Vector3,
};
use anyhow::{anyhow, Result};
use rand::{prelude::StdRng, Rng, SeedableRng};
use std::collections::HashMap;

pub const NUM_WEARS: u64 = 3;
pub const MAX_NUM_EAT: u64 = 5;
pub trait Entity {
    fn inventory(&self) -> &Inventory;
    fn drops(&self) -> &Inventory;
    fn equipped(&self) -> &Inventory;
    fn worn(&self) -> &Inventory;
    fn stats(&self) -> &Stat;
    fn loc(&self) -> &Vector3;
    fn abilities(&self) -> HashMap<String, Ability>;
    fn xp(&self) -> i64;
    fn name(&self) -> String;
    fn id(&self) -> ID;

    fn inventory_mut(&mut self) -> &mut Inventory;
    fn drops_mut(&mut self) -> &mut Inventory;
    fn equipped_mut(&mut self) -> &mut Inventory;
    fn worn_mut(&mut self) -> &mut Inventory;
    fn stats_mut(&mut self) -> &mut Stat;
    fn loc_mut(&mut self) -> &mut Vector3;
    fn set_xp(&mut self, xp: i64);
    fn send_text(&mut self, str: String);
    fn send_display(&mut self, img: Image);
    fn send_image(&mut self, s: String);
    fn rng(&mut self) -> &mut StdRng;

    fn entrance(&mut self) -> Result<String>;
    fn attack(&mut self) -> Result<String>;
    fn run(&mut self) -> Result<String>;
    fn victory(&mut self) -> Result<String>;
    fn loss(&mut self) -> Result<String>;

    fn equip(&mut self, item_name: &ItemName, g: &GameData) -> Result<()> {
        let item = g
            .items
            .get(item_name)
            .ok_or(anyhow!(format!("invalid item name! {:?}", item_name)))?;

        if !item.equipable {
            return Err(anyhow!("you cannot equip this item"));
        }

        self.inventory_mut().change(item_name.clone(), -1)?;
        self.dequip();
        self.equipped_mut().add(item_name.clone(), 1);
        Ok(())
    }

    fn dequip(&mut self) {
        let equipped_item = self.equipped().items().cloned().next();
        if let Some(equipped_item) = equipped_item {
            self.inventory_mut().add(equipped_item, 1);
            self.equipped_mut().clear();
        }
    }

    fn unwear(&mut self, item_name: &ItemName, g: &GameData) -> Result<()> {
        if self.worn().get(item_name) > 0 {
            self.worn_mut().change(item_name.clone(), -1)?;
            self.inventory_mut().add(item_name.clone(), 1);
            let item = g
                .items
                .get(item_name)
                .ok_or(anyhow!(format!("invalid item name! {:?}", item_name)))?;
            self.stats_mut().remove_buffs(&item.buffs.stat_buffs, g);
            Ok(())
        } else {
            Err(anyhow!(format!(
                "cannot unwear {:?} because you aren't wearing it!",
                item_name
            )))
        }
    }

    fn unwear_all(&mut self, g: &GameData) -> Result<()> {
        let worn_items: Vec<ItemName> = self.worn().items().cloned().collect();
        for item in worn_items {
            for _ in 0..self.worn().get(&item) {
                self.unwear(&item, g)?;
            }
        }
        Ok(())
    }

    fn wear(&mut self, item_name: &ItemName, g: &GameData) -> Result<()> {
        self.inventory_mut().change(item_name.clone(), -1)?;
        if self.worn().size() < NUM_WEARS {
            let item = g
                .items
                .get(item_name)
                .ok_or(anyhow!(format!("invalid item name! {:?}", item_name)))?;
            if !item.wearable {
                return Err(anyhow!("you cannot wear this item!"));
            }
            self.stats_mut().add_buffs(&item.buffs.stat_buffs, g);
            self.worn_mut().add(item_name.clone(), 1);
            Ok(())
        } else {
            Err(anyhow!(format!(
                "cannot wear more than {} items!",
                NUM_WEARS
            )))
        }
    }

    fn run_ability(
        &mut self,
        opponent: Option<Box<&mut dyn Entity>>,
        battle_map: &mut BattleMap,
        ability: Ability,
        item: Option<ItemName>,
        g: &GameData,
    ) -> Result<()> {
        // checks to ensure we can actually use the ability
        if self.xp() + ability.xp_cost < 0 {
            return Err(anyhow!(format!(
                "you need at least {} xp to use this ability",
                ability.xp_cost
            )));
        }
        if self.stats().energy() + ability.energy < 0.0 {
            return Err(anyhow!(format!(
                "you need at least {} energy to use this ability",
                -ability.energy
            )));
        }
        if !self.inventory().contains(&ability.require_items) {
            return Err(anyhow!(format!(
                "you need the following items to use this ability: {:?}",
                ability.require_items
            )));
        }

        // if we're in battle, do damage calcs
        if let Some(opponent) = opponent {
            opponent.send_text(format!(
                "{} used '{}'",
                self.name(),
                ability.name
            ));
            if let Some(item) = item {
                opponent.send_text(format!(" which is an ability of {:?}.\n", item));
            } else {
                opponent.send_text(".\n".into());
            }

            battle_map.add_effect(
                opponent.id(),
                StatusEffect::Damage(ability.damage),
                ability.repeat as usize + 1,
            )?;

            battle_map.add_effect(
                self.id(),
                StatusEffect::Block(ability.block),
                ability.repeat as usize + 1,
            )?;

            battle_map.add_effect(
                self.id(),
                StatusEffect::Counter(ability.counter),
                ability.repeat as usize + 1,
            )?;
        }

        self.set_xp(self.xp() - ability.xp_cost);
        self.stats_mut().change_energy(ability.energy, g)?;
        self.stats_mut().change_health(ability.health, g);
        self.inventory_mut().remove_items(&ability.remove_items)?;
        self.inventory_mut().add_items(&ability.make_items);
        if ability.destroy_item {
            self.equipped_mut().clear();
        }
        Ok(())
    }

    fn eat(
        &mut self,
        opponent: Option<Box<&mut dyn Entity>>,
        battle_map: &mut BattleMap,
        item_name: &ItemName,
        amount: u64,
        g: &GameData,
    ) -> Result<()> {
        if amount > MAX_NUM_EAT {
            return Err(anyhow!(format!(
                "you cannot eat more than {} items",
                MAX_NUM_EAT
            )));
        }
        let item = g
            .items
            .get(item_name)
            .ok_or(anyhow!(format!("invalid item name! {:?}", item_name)))?;

        if let Some(ability) = item.abilities.get("eat") {
            self.inventory_mut()
                .change(item_name.clone(), -(amount as i64))?;

            if let Some(opponent) = opponent {
                for _ in 0..amount {
                    let ability = ability.clone();
                    self.run_ability(
                        Some(Box::new(*opponent)),
                        battle_map,
                        ability,
                        Some(item.name.clone()),
                        g,
                    )?;
                }
            } else {
                for _ in 0..amount {
                    let ability = ability.clone();
                    self.run_ability(None, battle_map, ability, Some(item.name.clone()), g)?;
                }
            }
            Ok(())
        } else {
            Err(anyhow!("you cannot eat this item"))
        }
    }

    fn do_random_ability(
        &mut self,
        opponent: Option<Box<&mut dyn Entity>>,
        item: Option<ItemName>,
        battle_map: &mut BattleMap,
        abilities: &HashMap<String, Ability>,
        g: &GameData,
        rng: &mut StdRng,
    ) -> Result<()> {
        let ability_names: Vec<&String> = abilities.keys().collect();
        if ability_names.len() > 0 {
            let rand_ability = abilities
                .get(ability_names[rng.gen_range(0, ability_names.len())])
                .expect("bug")
                .clone();
            self.run_ability(opponent, battle_map, rand_ability, item, g)
        } else {
            Err(anyhow!("no abilities to do"))
        }
    }

    fn try_random_move(
        &mut self,
        opponent: Option<Box<&mut dyn Entity>>,
        battle_map: &mut BattleMap,
        g: &GameData,
        rng: &mut StdRng,
    ) -> Result<()> {
        let random_move: f64 = rng.gen();
        if random_move < 0.2 {
            // try to equip a random item
            let filter = |x: &Item| x.equipable;
            let item = get_items_rand(&self.inventory(), 1, filter, g, rng)?;
            if let Some(item_name) = item.get(0) {
                if let Some(opp) = opponent {
                    opp.send_text(format!("your opponent equipped {:?}\n", item_name));
                }
                self.equip(item_name, g)
            } else {
                Err(anyhow!("no items to equip"))
            }
        } else if random_move < 0.4 {
            // attempt to do a randomly chosen ability of the current item
            if let Some(item_name) = self.equipped().items().next() {
                let item = g
                    .items
                    .get(item_name)
                    .ok_or(anyhow!(format!("invalid item name! {:?}", item_name)))?;
                let item_name = item_name.clone();
                self.do_random_ability(
                    opponent,
                    Some(item_name),
                    battle_map,
                    &item.abilities,
                    g,
                    rng,
                )
            } else {
                Err(anyhow!("no item equipped"))
            }
        } else if random_move < 0.6 {
            // attempt to do a random inherent ability
            self.do_random_ability(opponent, None, battle_map, &self.abilities(), g, rng)
        } else if random_move < 0.99 {
            // attempt to eat a random amount of a random item
            let filter = |x: &Item| x.abilities.contains_key("eat");
            let item = get_all_items(&self.inventory(), filter, g)?;
            if item.len() > 0 {
                let rand_item = &item[rng.gen_range(0, item.len())];
                let count = self.inventory().get(rand_item);
                let num_eat = rng.gen_range(1, MAX_NUM_EAT + 1).min(count);
                self.eat(opponent, battle_map, rand_item, num_eat, g)
            } else {
                Err(anyhow!("no items to eat"))
            }
        } else {
            // pass turn, do nothing
            // this will always work
            // so we are probabilistically guaranteed termination
            if let Some(opponent) = opponent {
                opponent.send_text(format!(
                    "{} skips their turn\n",
                    self.name()
                ));
            }
            Ok(())
        }
    }

    // either equip a random equippable item
    // do a random ability of the currently equipped item
    // do a random ability
    // eat a random amount of a randomly chosen item in the inventory
    fn do_random_move(
        &mut self,
        opponent: Option<Box<&mut dyn Entity>>,
        battle_map: &mut BattleMap,
        g: &GameData,
    ) {
        let mut rng: StdRng = SeedableRng::seed_from_u64(self.rng().gen());
        let mut res = Err(anyhow!("start"));
        if let Some(opponent) = opponent {
            while res.is_err() {
                res = self.try_random_move(Some(Box::new(*opponent)), battle_map, g, &mut rng);
            }
        } else {
            while res.is_err() {
                res = self.try_random_move(None, battle_map, g, &mut rng);
            }
        }
    }
}

fn mul(a: &HashMap<DmgType, f64>, b: &HashMap<DmgType, f64>) -> HashMap<DmgType, f64> {
    let mut new = HashMap::new();
    for (k, v) in a {
        new.insert(k.clone(), (*v) * (*b.get(k).expect("bug")));
    }
    new
}

pub fn get_items_rand(
    inventory: &Inventory,
    num_items: u64,
    filter: fn(&Item) -> bool,
    g: &GameData,
    rng: &mut StdRng,
) -> Result<Vec<ItemName>> {
    let mut inventory = inventory.clone();
    let mut returned_items = Vec::new();
    for _ in 0..num_items {
        let mut usable_items = Vec::new();
        let mut usable_summed = Vec::new();
        let mut usable_sum = 0;
        for item_name in inventory.items() {
            let item = g
                .items
                .get(item_name)
                .ok_or(anyhow!(format!("invalid item name! {:?}", item_name)))?;
            if filter(item) {
                usable_sum += inventory.get(item_name);
                usable_summed.push(usable_sum);
                usable_items.push(item_name.clone());
            }
        }
        if usable_sum == 0 {
            break;
        }
        let index = rng.gen_range(0, usable_sum);
        for i in 0..usable_summed.len() {
            if usable_summed[i] > index {
                returned_items.push(usable_items[i].clone());
                inventory.change(usable_items[i].clone(), -1)?;
                break;
            }
        }
    }
    
    return Ok(returned_items);
}

pub fn get_all_items(
    inventory: &Inventory,
    filter: fn(&Item) -> bool,
    g: &GameData,
) -> Result<Vec<ItemName>> {
    let mut filtered_items = Vec::new();
    for item_name in inventory.items() {
        let item = g
            .items
            .get(item_name)
            .ok_or(anyhow!(format!("invalid item name! {:?}", item_name)))?;
        if filter(item) {
            filtered_items.push(item_name.clone());
        }
    }
    return Ok(filtered_items);
}
