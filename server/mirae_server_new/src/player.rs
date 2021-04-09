use crate::{
    combat::ID,
    display::Image,
    entity::Entity,
    gamedata::{
        gamedata::{GameData, StatType},
        item::Ability,
    },
    inventory::Inventory,
    playerout::PlayerOut,
    stat::Stat,
    vector3::Vector3,
};
use rand::{prelude::StdRng, thread_rng, SeedableRng, Rng};
use std::{collections::HashMap, sync::mpsc::Sender};
use anyhow::Result;

pub struct Player {
    id: usize,
    inventory: Inventory,
    drops: Inventory,
    equip: Inventory,
    wear: Inventory,
    stats: Stat,
    loc: Vector3,
    rng: StdRng,
    xp: i64,
    pub sender: Sender<(PlayerOut, Option<usize>)>,
}

impl Player {
    pub fn new(
        id: usize,
        sender: Sender<(PlayerOut, Option<usize>)>,
        g: &GameData,
        rng: &mut StdRng
    ) -> Result<Self> {
        let mut base_stats= HashMap::new();
        base_stats.insert(StatType::from("max_health".to_string()), 10.0);
        base_stats.insert(StatType::from("max_energy".to_string()), 3.0);
        base_stats.insert(StatType::from("speed".to_string()), 1.0);
        Ok(Player {
            id,
            sender,
            inventory: Inventory::new(),
            drops: Inventory::new(),
            equip: Inventory::new(),
            wear: Inventory::new(),
            stats: Stat::new(base_stats, &g.stat)?,
            loc: Vector3::new(0, 0, 0),
            rng: SeedableRng::seed_from_u64(rng.gen()),
            xp: 1000,
        })
    }
}

impl Entity for Player {
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
        HashMap::new()
    }

    fn xp(&self) -> i64 {
        self.xp
    }

    fn name(&self) -> Option<String> {
        None
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

    fn send_image(&mut self, _: Image) {
        todo!()
    }
    fn send_text(&mut self, _: String) {
        todo!()
    }

    fn id(&self) -> ID {
        ID::player(self.id)
    }
}
