use crate::{
    entity::Entity,
    gamedata::gamedata::{DmgType, GameData},
    stat::default_empty_fields,
};
use anyhow::{anyhow, Result};
use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub enum EntityType {
    Mob,
    Player,
}

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct ID {
    pub id: usize,
    pub enity_type: EntityType,
}

impl ID {
    pub fn player(id: usize) -> Self {
        Self {
            id,
            enity_type: EntityType::Player,
        }
    }

    pub fn mob(id: usize) -> Self {
        Self {
            id,
            enity_type: EntityType::Mob,
        }
    }
}

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
struct BattleHandle(usize);

pub struct BattleMap {
    id_to_handle: HashMap<ID, BattleHandle>,
    handle_to_ids: HashMap<BattleHandle, (ID, ID)>,
    handle_to_data: HashMap<BattleHandle, BattleData>,
    curr_handle: BattleHandle,
}

pub struct BattleData {
    id_to_data: HashMap<ID, CombatData>,
    defense_turn: bool,
}

impl BattleData {
    pub fn opponent_combat_data(&self, id: ID) -> Result<&CombatData> {
        for (k, v) in &self.id_to_data {
            if &id != k {
                return Ok(v);
            }
        }
        Err(anyhow!("bad battledata struct!"))
    }

    pub fn combat_data(&self, id: ID) -> Result<&CombatData> {
        self.id_to_data
            .get(&id)
            .ok_or(anyhow!("bad battledata struct!"))
    }

    pub fn opponent_combat_data_mut(&mut self, id: ID) -> Result<&mut CombatData> {
        for (k, v) in &mut self.id_to_data {
            if &id != k {
                return Ok(v);
            }
        }
        Err(anyhow!("bad battledata struct!"))
    }

    pub fn combat_data_mut(&mut self, id: ID) -> Result<&mut CombatData> {
        self.id_to_data
            .get_mut(&id)
            .ok_or(anyhow!("bad battledata struct!"))
    }
}

pub struct CombatData {
    pub acc_speed: f64,
    pub stunned: bool,
    pub status_effects: Vec<(StatusEffect, usize)>,
}

#[derive(Clone)]
pub enum StatusEffect {
    Stun,
    Damage(HashMap<DmgType, f64>),
    Block(HashMap<DmgType, f64>),
    Counter(HashMap<DmgType, f64>),
}

impl BattleMap {
    pub fn new() -> Self {
        BattleMap {
            id_to_handle : HashMap::new(),
            handle_to_ids : HashMap::new(),
            handle_to_data : HashMap::new(),
            curr_handle : BattleHandle(0)
        }
    }

    fn get_handle(&self, id: ID) -> Result<&BattleHandle> {
        self.id_to_handle
            .get(&id)
            .ok_or(anyhow!("this entity isn't fighting anything"))
    }

    pub fn get_battle_data(&self, id: ID) -> Result<&BattleData> {
        let handle = self.get_handle(id)?;
        self.handle_to_data
            .get(handle)
            .ok_or(anyhow!("huh? bad handle?"))
    }

    pub fn get_battle_data_mut(&mut self, id: ID) -> Result<&mut BattleData> {
        let handle = self.get_handle(id)?.clone();
        self.handle_to_data
            .get_mut(&handle)
            .ok_or(anyhow!("huh? bad handle?"))
    }

    pub fn get_opponent(&self, id: ID) -> Result<ID> {
        let handle = self.get_handle(id)?;
        let (id1, id2) = self
            .handle_to_ids
            .get(handle)
            .ok_or(anyhow!("bad battle handle"))?;
        if id1 == &id {
            Ok(id2.clone())
        } else {
            Ok(id1.clone())
        }
    }

    pub fn add_effect(&mut self, id: ID, effect: StatusEffect, num_turns: usize) -> Result<()> {
        let battle_data = self.get_battle_data_mut(id)?;
        let id_cd = battle_data.combat_data_mut(id)?;
        id_cd.status_effects.push((effect, num_turns));
        Ok(())
    }

    pub fn turn(&self, id: ID) -> Result<bool> {
        let battle_data = self.get_battle_data(id)?;
        let id_cd = battle_data.combat_data(id)?;
        let op_cd = battle_data.opponent_combat_data(id)?;
        Ok(id_cd.acc_speed > op_cd.acc_speed)
    }

    pub fn init_battle(
        &mut self,
        attacker: Box<&mut dyn Entity>,
        defender: Box<&mut dyn Entity>,
        g: &GameData,
    ) -> Result<()> {
        let battle_handle = self.curr_handle;
        self.curr_handle = BattleHandle(battle_handle.0 + 1);
        self.id_to_handle.insert(attacker.id(), battle_handle);
        self.id_to_handle.insert(defender.id(), battle_handle);
        self.handle_to_ids
            .insert(battle_handle, (attacker.id(), defender.id()));

        let speed = "speed";
        let attacker_cd = CombatData {
            acc_speed: attacker.stats().get(speed, g)?,
            stunned: false,
            status_effects: Vec::new(),
        };
        let defender_cd = CombatData {
            acc_speed: defender.stats().get(speed, g)?,
            stunned: false,
            status_effects: Vec::new(),
        };

        let mut id_to_data = HashMap::new();
        id_to_data.insert(attacker.id(), attacker_cd);
        id_to_data.insert(defender.id(), defender_cd);
        let battle_data = BattleData {
            id_to_data,
            defense_turn: false,
        };

        self.handle_to_data.insert(battle_handle, battle_data);
        Ok(())
    }

    pub fn end_battle(&mut self, id: ID) -> Result<()> {
        let opponent = self.get_opponent(id)?;
        let handle = self.get_handle(id)?.clone();
        self.handle_to_ids.remove(&handle);
        self.handle_to_data.remove(&handle);
        self.id_to_handle.remove(&id);
        self.id_to_handle.remove(&opponent);
        Ok(())
    }

    fn handle_status_effects(&mut self, entity: Box<&mut dyn Entity>, g: &GameData) -> Result<()> {
        let opponent = self.get_opponent(entity.id())?;
        let battle_data = self.get_battle_data(entity.id())?;
        let combat_data = battle_data.combat_data(entity.id())?;
        let mut net_dmg = default_empty_fields(&HashMap::new(), 0.0, &g.dmg);
        let status_effects = combat_data.status_effects.clone();
        for (se, _) in &status_effects {
            match se {
                StatusEffect::Damage(dmg) => {
                    net_dmg = add(&net_dmg, dmg);
                }
                _ => {}
            }
        }

        for (se, _) in &status_effects {
            match se {
                StatusEffect::Counter(dmg) => {
                    let counter_damage = mul(dmg, &net_dmg);
                    self.add_effect(opponent, StatusEffect::Damage(counter_damage), 1)?;
                }
                _ => {}
            }
        }

        for (se, _) in &status_effects {
            match se {
                StatusEffect::Block(dmg) => {
                    net_dmg = mul(&net_dmg, dmg);
                }
                _ => {}
            }
        }

        let mut total_dmg = 0.0;
        for (dmg_type, val) in &net_dmg {
            if val.abs() > f64::EPSILON {
                entity.send_text(format!("you recieved {} {:?} damage.\n", val, dmg_type));
            }
            total_dmg += val;
        }

        if total_dmg.abs() > f64::EPSILON {
            entity.send_text(format!("you recieved {} damage in total.\n", total_dmg));
        }

        let mut stunned = false;
        for (se, _) in &status_effects {
            match se {
                StatusEffect::Stun => {
                    stunned = true;
                }
                _ => {}
            }
        }

        let battle_data = self.get_battle_data_mut(entity.id())?;
        let combat_data = battle_data.combat_data_mut(entity.id())?;

        if stunned {
            entity.send_text("you are stunned!\n".into());
        }
        if combat_data.stunned && !stunned {
            entity.send_text("you are no longer stunned!\n".into());
        }

        combat_data.stunned = stunned;

        for (_, num_turns) in &mut combat_data.status_effects {
            *num_turns -= 1;
        }
        combat_data
            .status_effects
            .retain(|(_, num_turns)| *num_turns != 0);
        Ok(())
    }

    pub fn do_turn(
        &mut self,
        a: Box<&mut dyn Entity>,
        b: Box<&mut dyn Entity>,
        g: &GameData,
    ) -> Result<()> {
        let speed = "speed";
        let a_speed = a.stats().get(speed, g)?;
        let b_speed = b.stats().get(speed, g)?;
        let a_turn = self.turn(a.id())?;

        let battle_data = self.get_battle_data_mut(a.id())?;

        if !battle_data.defense_turn {
            if a_turn && !battle_data.combat_data(b.id())?.stunned {
                battle_data.combat_data_mut(b.id())?.acc_speed += b_speed;
            } else if !battle_data.combat_data(a.id())?.stunned {
                battle_data.combat_data_mut(a.id())?.acc_speed += a_speed;
            }
            let new_turn = self.turn(a.id())?;

            let battle_data = self.get_battle_data_mut(a.id())?;
            let combat_data = battle_data.combat_data_mut(a.id())?;

            
            for (se, num_turns) in &mut combat_data.status_effects {
                match se {
                    StatusEffect::Stun => {
                        *num_turns -= 1;
                    }
                    _ => {}
                }
            }
            combat_data
                .status_effects
                .retain(|(_, num_turns)| *num_turns != 0);

            battle_data.defense_turn = a_turn ^ new_turn;
        } else {
            battle_data.defense_turn = false;
            if a_turn {
                self.handle_status_effects(a, g)?;
            } else {
                self.handle_status_effects(b, g)?;
            }
        }
        Ok(())
    }
}

fn mul(a: &HashMap<DmgType, f64>, b: &HashMap<DmgType, f64>) -> HashMap<DmgType, f64> {
    let mut new = HashMap::new();
    for (k, v) in a {
        new.insert(k.clone(), (*v) * (*b.get(k).expect("bug")));
    }
    new
}

fn add(a: &HashMap<DmgType, f64>, b: &HashMap<DmgType, f64>) -> HashMap<DmgType, f64> {
    let mut new = HashMap::new();
    for (k, v) in a {
        new.insert(k.clone(), (*v) + (*b.get(k).expect("bug")));
    }
    new
}

/*
pub struct BattleMap(Vec<BattleData>);

pub struct BattleData {
    pub attacker: CombatData,
    pub defender: CombatData,
    pub attacker_turn: bool,
}

impl BattleData {
    pub fn new(attacker: ID, attacker_speed: f64, defender: ID, defender_speed: f64) -> Self {
        Self {
            attacker: CombatData {
                id: attacker,
                status_effects: Vec::new(),
                accumulated_speed: attacker_speed,
            },
            defender: CombatData {
                id: defender,
                status_effects: Vec::new(),
                accumulated_speed: defender_speed,
            },
            attacker_turn: attacker_speed > defender_speed,
        }
    }

    pub fn turn(&self, entity: ID) -> bool {
        (entity == self.attacker.id && self.attacker_turn)
            || (entity == self.defender.id && !self.attacker_turn)
    }

    pub fn opponent(&self, entity: ID) -> Option<ID> {
        if entity == self.attacker.id {
            Some(self.defender.id)
        } else if entity == self.defender.id {
            Some(self.attacker.id)
        } else {
            None
        }
    }

    pub fn combat_data_mut(&mut self, entity: ID) -> Option<&mut CombatData> {
        if entity == self.attacker.id {
            Some(&mut self.attacker)
        } else if entity == self.defender.id {
            Some(&mut self.defender)
        } else {
            None
        }
    }
}

pub struct CombatData {
    pub id: ID,
    pub accumulated_speed: f64,
    pub status_effects: Vec<(StatusEffect, usize)>,
}

#[derive(Clone)]
pub enum StatusEffect {
    Stun,
    Damage(HashMap<DmgType, f64>),
    Block(HashMap<DmgType, f64>),
}

impl BattleMap {
    pub fn battle_data_mut(&mut self, entity: ID) -> Option<&mut BattleData> {
        for data in &mut self.0 {
            if data.attacker.id == entity || data.defender.id == entity {
                return Some(data);
            }
        }
        None
    }

    pub fn battle_data(&self, entity: ID) -> Option<&BattleData> {
        for data in &self.0 {
            if data.attacker.id == entity || data.defender.id == entity {
                return Some(data);
            }
        }
        None
    }

    pub fn begin_battle(
        &mut self,
        attacker: ID,
        defender: ID,
        world: &World,
        players: &Vec<Option<Player>>,
        g: &GameData,
    ) -> Result<()> {
        for data in &self.0 {
            if data.attacker.id == attacker
                || data.attacker.id == defender
                || data.defender.id == attacker
                || data.defender.id == defender
            {
                return Err(anyhow!("cannot initiate battle, one of those entities is already fighting something else!"));
            }
        }
        let speed = "speed";
        let (attacker_speed, defender_speed) = match (attacker.enity_type, defender.enity_type) {
            (EntityType::Mob, EntityType::Mob) => {
                let attacker_mob = world.get_mob(attacker.id)?;
                let defender_mob = world.get_mob(attacker.id)?;
                (
                    attacker_mob.stats().get(speed, g)?,
                    defender_mob.stats().get(speed, g)?,
                )
            }
            (EntityType::Mob, EntityType::Player) => {
                let attacker_mob = world.get_mob(attacker.id)?;
                let defender_player = players[defender.id]
                    .as_ref()
                    .ok_or(anyhow!("invalid player id"))?;
                (
                    attacker_mob.stats().get(speed, g)?,
                    defender_player.stats().get(speed, g)?,
                )
            }
            (EntityType::Player, EntityType::Mob) => {
                let attacker_player = players[attacker.id]
                    .as_ref()
                    .ok_or(anyhow!("invalid player id"))?;
                let defender_mob = world.get_mob(defender.id)?;
                (
                    attacker_player.stats().get(speed, g)?,
                    defender_mob.stats().get(speed, g)?,
                )
            }
            (EntityType::Player, EntityType::Player) => {
                let (attacker_player, defender_player) =
                    get_two(attacker.id, defender.id, players)?;
                (
                    attacker_player.stats().get(speed, g)?,
                    defender_player.stats().get(speed, g)?,
                )
            }
        };
        self.0.push(BattleData::new(
            attacker,
            attacker_speed,
            defender,
            defender_speed,
        ));
        Ok(())
    }

    pub fn end_battle(&mut self, entity: ID) -> Result<()> {
        for i in 0..self.0.len() {
            if self.0[i].attacker.id == entity || self.0[i].defender.id == entity {
                self.0.remove(i);
                return Ok(());
            }
        }
        Err(anyhow!("the entity provided isn't battling anything"))
    }

    pub fn do_turn(&mut self, entity: Box<&mut dyn Entity>, opponent: Box<&mut dyn Entity>, g : &GameData) -> Result<()> {
        let entity_combat_data = self
            .battle_data_mut(entity.id())
            .expect("bug")
            .combat_data_mut(entity.id())
            .expect("bug");

        let opponent_combat_data = self
            .battle_data_mut(opponent.id())
            .expect("bug")
            .combat_data_mut(opponent.id())
            .expect("bug");





        opponent_combat_data.accumulated_speed += opponent.stats().get("speed", g)?;

        let mut turn = entity_combat_data.accumulated_speed > opponent_combat_data.accumulated_speed;
        let battle_data = self.battle_data_mut(entity.id()).expect("bug");

        if entity.id() == battle_data.defender.id {
            turn = !turn;
        }

        self.battle_data_mut(entity.id()).expect("bug").attacker_turn = turn;




        Ok(())
    }
}
*/
