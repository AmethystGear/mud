use crate::{
    combat::ID,
    display::Image,
    entity::Entity,
    gamedata::{
        gamedata::{DmgType, GameData, ItemName, Named, StatType},
        item::Ability,
    },
    inventory::Inventory,
    playerout::PlayerOut,
    stat::{default_empty_fields, Stat},
    vector3::Vector3,
    world::World,
};
use anyhow::{anyhow, Result};
use crossbeam::channel::Sender;
use rand::{prelude::StdRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerSave {
    inventory: Inventory,
    equip: Inventory,
    wear: Inventory,
    stats: Stat,
    xp: i64,
}

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
    attack_buffs: HashMap<DmgType, f64>,
    defense_buffs: HashMap<DmgType, f64>,
    pub return_posn: Vector3,
    pub sender: Sender<(PlayerOut, Option<usize>)>,
    pub username: Option<String>,
}

impl Player {
    pub fn new(
        id: usize,
        sender: Sender<(PlayerOut, Option<usize>)>,
        g: &GameData,
        rng: &mut StdRng,
    ) -> Result<Self> {
        let mut base_stats = HashMap::new();
        base_stats.insert(StatType::from("max_health".to_string()), 10.0);
        base_stats.insert(StatType::from("max_energy".to_string()), 10.0);
        base_stats.insert(StatType::from("speed".to_string()), 10.0);
        let mut inventory = Inventory::new();
        inventory.set(ItemName::checked_from("stick".into(), g)?, 1);
        inventory.set(ItemName::checked_from("workbench".into(), g)?, 1);

        let buffs = default_empty_fields(&HashMap::new(), 1.0, &g.dmg);

        Ok(Player {
            id,
            sender,
            inventory,
            drops: Inventory::new(),
            equip: Inventory::new(),
            wear: Inventory::new(),
            stats: Stat::new(base_stats, &g.stat)?,
            loc: Vector3::new(0, 0, 0),
            rng: SeedableRng::seed_from_u64(rng.gen()),
            xp: 1000,
            return_posn: Vector3::zero(),
            username: None,
            attack_buffs: buffs.clone(),
            defense_buffs: buffs,
        })
    }

    pub fn save(&self) -> Result<String> {
        let save = PlayerSave {
            inventory: self.inventory.clone(),
            equip: self.equip.clone(),
            wear: self.wear.clone(),
            stats: self.stats.clone(),
            xp: self.xp,
        };
        Ok(serde_jacl::ser::to_string(&save)?)
    }

    pub fn load(&mut self, s: String) -> Result<()> {
        let save: PlayerSave = serde_jacl::de::from_str(&s)?;
        self.inventory = save.inventory;
        self.equip = save.equip;
        self.wear = save.wear;
        self.stats = save.stats;
        self.xp = save.xp;
        Ok(())
    }

    pub fn respawn(&mut self, world: &World, g: &GameData) -> Result<()> {
        self.send_text(format!("respawning...\n"));
        self.stats_mut().reset_health(&g);
        self.stats_mut().reset_energy(&g);
        let dim = world.blocks().dim;
        let posn;
        loop {
            let new_posn = Vector3::new(
                self.rng.gen_range(0, dim.x()),
                self.rng.gen_range(0, dim.y()),
                self.rng.gen_range(0, dim.z()),
            );
            if !world.get_block_at(g, new_posn)?.solid
                && world.get_mobtemplate_at(new_posn, g).is_err()
            {
                posn = new_posn;
                break;
            }
        }
        self.loc_mut().set(posn);
        self.return_posn = posn;
        Ok(())
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

    fn name(&self) -> String {
        if let Some(name) = &self.username {
            name.clone()
        } else {
            format!("player {}", self.id)
        }
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

    fn send_display(&mut self, i: Image, static_display : bool) {
        let mut p_out = PlayerOut::new();
        if static_display {
            p_out.append_static_display(i);
        } else {
            p_out.append_display(i);
        }
        self.sender.send((p_out, None)).unwrap();
    }

    fn send_text(&mut self, s: String) {
        let mut p_out = PlayerOut::new();
        p_out.append_text(s);
        self.sender.send((p_out, None)).unwrap();
    }

    fn send_image(&mut self, s: String) {
        let mut p_out = PlayerOut::new();
        p_out.append_img(s);
        self.sender.send((p_out, None)).unwrap();
    }

    fn id(&self) -> ID {
        ID::player(self.id)
    }

    fn entrance(&mut self) -> Result<String> {
        Err(anyhow!("no quote"))
    }

    fn attack(&mut self) -> Result<String> {
        Err(anyhow!("no quote"))
    }

    fn run(&mut self) -> Result<String> {
        Err(anyhow!("no quote"))
    }

    fn victory(&mut self) -> Result<String> {
        Err(anyhow!("no quote"))
    }

    fn loss(&mut self) -> Result<String> {
        Err(anyhow!("no quote"))
    }

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
