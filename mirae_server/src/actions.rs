use crate::{
    combat::{BattleMap, EntityType, ID},
    display::{Bounds, Image},
    entity::{Entity, MAX_NUM_EAT},
    gamedata::{
        block::Block,
        gamedata::{BlockName, GameData, ItemName, MobName, Named},
        mobtemplate::MobTemplate,
    },
    player::Player,
    players_op,
    vector3::Vector3,
    world::World,
    PLAYER_SAVE_FOLDER,
};
use anyhow::{anyhow, Result};
use fs::File;
use rand::{thread_rng, Rng};
use serde_jacl::structs::{Literal, Number};
use std::{
    collections::{HashSet, VecDeque},
    fs,
    io::Write,
    path::Path,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

const BAD_ARGS: &str =
    "bad arguments to command, try help <command> to see how to use it correctly";

pub fn dispatch(data: ActionData) -> Result<()> {
    match data.params.get(0) {
        Some(Literal::String(s)) => {
            let func = match s.as_str() {
                "disp" => disp,
                "inv" | "inventory" => inv,
                "eat" | "drink" | "consume" => eat,
                "w" | "a" | "s" | "d" | "ww" | "aa" | "ss" | "dd" | "q" | "e" => step,
                "wear" | "unwear" | "wearing" => wear,
                "equip" | "dequip" | "equipped" => equip,
                "use" => use_ability,
                "upgrade" => upgrade,
                "battle" => battle,
                "map" => map,
                "pass" => pass,
                "run" | "flee" => run,
                "acc" | "account" => account,
                "scan" => scan,
                "info" | "stat" => info,
                _ => return Err(anyhow!("invalid command")),
            };
            func(data)
        }
        _ => Err(anyhow!("expected string as first parameter")),
    }
}

fn get_player_and_opponent<'a>(
    opponent: ID,
    players: Arc<Vec<Arc<RwLock<Option<Player>>>>>,
    player_id: usize,
    world: &'a mut World,
) -> Result<(&'a mut Player, Box<&'a mut dyn Entity>)> {
    match opponent.enity_type {
        EntityType::Mob => {
            let player = get_mut(players, player_id)?;
            let opponent = world.get_mob_mut(opponent.id)?;
            Ok((
                player.as_mut().ok_or(anyhow!("invalid player id"))?,
                Box::new(opponent as &mut dyn Entity),
            ))
        }
        EntityType::Player => {
            let (player, opponent) = get_two_mut(player_id, opponent.id, players)?;
            Ok((
                player.as_mut().ok_or(anyhow!("invalid player id"))?,
                Box::new(opponent.as_mut().ok_or(anyhow!("invalid player id"))? as &mut dyn Entity),
            ))
        }
    }
}

pub fn get_mut<'a>(
    players: Arc<Vec<Arc<RwLock<Option<Player>>>>>,
    player_id: usize,
) -> Result<RwLockWriteGuard<'a, Option<Player>>> {
    players[player_id].write().map_err(players_op)
}

pub fn get<'a>(
    players: Arc<Vec<Arc<RwLock<Option<Player>>>>>,
    player_id: usize,
) -> Result<RwLockReadGuard<'a, Option<Player>>> {
    players[player_id].read().map_err(players_op)
}

pub fn get_two_mut<'a>(
    a: usize,
    b: usize,
    players: Arc<Vec<Arc<RwLock<Option<Player>>>>>,
) -> Result<(
    RwLockWriteGuard<'a, Option<Player>>,
    RwLockWriteGuard<'a, Option<Player>>,
)> {
    let split: _ = players.split_at(std::cmp::max(a, b) as usize);
    let head: _ = split.0;
    let tail: _ = split.1;
    if a > b {
        Ok((
            tail[0].write().map_err(players_op)?,
            head[b as usize].write().map_err(players_op)?,
        ))
    } else if a < b {
        Ok((
            head[a as usize].write().map_err(players_op)?,
            tail[0].write().map_err(players_op)?,
        ))
    } else {
        Err(anyhow!("player ids cannot be equal"))
    }
}

pub fn get_two<'a>(
    a: usize,
    b: usize,
    players: Arc<Vec<Arc<RwLock<Option<Player>>>>>,
) -> Result<(
    RwLockReadGuard<'a, Option<Player>>,
    RwLockReadGuard<'a, Option<Player>>,
)> {
    let split: _ = players.split_at(std::cmp::max(a, b) as usize);
    let head: _ = split.0;
    let tail: _ = split.1;
    if a > b {
        Ok((
            tail[0].read().map_err(players_op)?,
            head[b as usize].read().map_err(players_op)?,
        ))
    } else if a < b {
        Ok((
            head[a as usize].read().map_err(players_op)?,
            tail[0].read().map_err(players_op)?,
        ))
    } else {
        Err(anyhow!("player ids cannot be equal"))
    }
}

pub struct ActionData<'a> {
    pub params: VecDeque<Literal>,
    pub player_id: usize,
    pub players: Arc<Vec<Arc<RwLock<Option<Player>>>>>,
    pub world: Arc<RwLock<World>>,
    pub battle_map: Arc<RwLock<BattleMap>>,
    pub g: &'a GameData,
}

fn run_turn_with(
    battle_map: &mut BattleMap,
    player_id: ID,
    players: Arc<Vec<Arc<RwLock<std::option::Option<Player>>>>>,
    world: &mut World,
    g: &GameData,
    func: &dyn Fn(
        &mut Player,
        Option<Box<&mut dyn Entity>>,
        &GameData,
        &mut BattleMap,
    ) -> Result<()>,
) -> Result<()> {
    if let Ok(opponent) = battle_map.get_opponent(player_id) {
        if battle_map.turn(opponent)? {
            Err(anyhow!("it's not your turn!"))
        } else {
            let (player, opp) = get_player_and_opponent(opponent, players, player_id.id, world)?;
            func(player, Some(opp), g, battle_map)?;
            let (player, opp) = get_player_and_opponent(opponent, players, player_id.id, world)?;
            battle_map.do_turn(Box::new(player), opp, g)
        }
    } else {
        let player = get_mut(players, player_id.id)?
            .as_mut()
            .ok_or(anyhow!("invalid player id"))?;
        func(player, None, g, battle_map)
    }
}

fn use_ability(mut data: ActionData) -> Result<()> {
    let player = get_mut(data.players, data.player_id)?
        .as_mut()
        .ok_or(anyhow!("invalid player id"))?;

    let ability_name;
    data.params.pop_front(); // ignore the first parameter
    match data.params.pop_front() {
        Some(Literal::String(s)) => {
            ability_name = s;
        }
        _ => return Err(anyhow!(BAD_ARGS)),
    }
    let item_name;
    if let Some(name) = player.equipped().items().next() {
        item_name = name;
    } else {
        return Err(anyhow!("you don't have any item equipped!"));
    }

    let item = data
        .g
        .items
        .get(item_name)
        .expect("player somehow equipped an item that doesn't exist");

    let item_name = item_name.clone();

    let ability;
    if let Some(a) = item.abilities.get(&ability_name) {
        ability = a;
    } else {
        return Err(anyhow!(format!(
            "the item {:?} doesn't have the ability {:?}. It has the abilities: {:?}",
            item_name,
            ability_name,
            item.abilities.keys()
        )));
    }

    let ability = ability.clone();
    let player_id = ID::player(data.player_id);
    let mut battle_map = data
        .battle_map
        .write()
        .map_err(|_| anyhow!("couldn't lock battle map"))?;
    let mut world = data
        .world
        .write()
        .map_err(|_| anyhow!("couldn't lock world"))?;

    let func = |player: &mut Player,
                opp: Option<Box<&mut dyn Entity>>,
                g: &GameData,
                battle_map: &mut BattleMap| {
        player.run_ability(opp, battle_map, ability.clone(), Some(item_name.clone()), g)
    };

    run_turn_with(
        &mut battle_map,
        player_id,
        data.players,
        &mut world,
        data.g,
        &func,
    )
}

fn eat(mut data: ActionData) -> Result<()> {
    let item_name;
    let amount;
    data.params.pop_front(); // ignore the first parameter
    match (data.params.pop_front(), data.params.pop_front()) {
        (Some(Literal::String(item)), Some(Literal::Number(Number::Int(number)))) => {
            item_name = ItemName::checked_from(item, data.g)?;
            if number > 0 {
                amount = number as u64;
            } else {
                return Err(anyhow!("can't eat a negative number of an item"));
            }
        }
        (Some(Literal::String(item)), None) => {
            item_name = ItemName::checked_from(item, data.g)?;
            amount = MAX_NUM_EAT;
        }
        _ => return Err(anyhow!(BAD_ARGS)),
    };
    let mut battle_map = data
        .battle_map
        .write()
        .map_err(|_| anyhow!("couldn't lock battle map"))?;
    let mut world = data
        .world
        .write()
        .map_err(|_| anyhow!("couldn't lock world"))?;
    let player_id = ID::player(data.player_id);

    let func =
        |player: &mut Player,
         opp: Option<Box<&mut dyn Entity>>,
         g: &GameData,
         battle_map: &mut BattleMap| { player.eat(opp, battle_map, &item_name, amount, g) };

    run_turn_with(
        &mut battle_map,
        player_id,
        data.players,
        &mut world,
        data.g,
        &func,
    )
}

enum Move {
    X(i64),
    Y(i64),
    // true = down (greater z values), false = up (lesser z values)
    Z(bool),
}

fn find_closest_posn(
    world: &World,
    g: &GameData,
    start: Vector3,
    filter: &dyn Fn(&Block, Option<&MobTemplate>) -> bool,
    max_dist: u64,
) -> Result<Vector3> {
    let closest = find_all_posn(world, g, start, filter, max_dist, 1)?;
    if closest.len() == 0 {
        Err(anyhow!("no such block in range"))
    } else {
        Ok(closest[0])
    }
}

fn find_all_posn(
    world: &World,
    g: &GameData,
    start: Vector3,
    filter: &dyn Fn(&Block, Option<&MobTemplate>) -> bool,
    max_dist: u64,
    max_ret: usize,
) -> Result<Vec<Vector3>> {
    let mut visited = HashSet::new();
    let mut to_eval = VecDeque::new();
    let mut ret = Vec::new();
    to_eval.push_back((start, 0));
    visited.insert(start);
    while let Some((curr, dist)) = to_eval.pop_front() {
        if ret.len() >= max_ret {
            break;
        }
        let block = world.get_block(g, world.blocks().index(curr)?)?;
        let mob_template = world.get_mobtemplate_at(curr, g).ok();
        if filter(block, mob_template) {
            ret.push(curr);
        } else if dist < max_dist {
            let neighbors = [
                Vector3::new(-1, 0, 0),
                Vector3::new(1, 0, 0),
                Vector3::new(0, -1, 0),
                Vector3::new(0, 1, 0),
                Vector3::new(-1, -1, 0),
                Vector3::new(1, 1, 0),
                Vector3::new(-1, 1, 0),
                Vector3::new(1, -1, 0),
            ];
            for neighbor in &neighbors {
                let neighbor = curr + *neighbor;
                if !visited.contains(&neighbor) && world.blocks().get(neighbor).is_ok() {
                    to_eval.push_back((neighbor, dist + 1));
                    visited.insert(neighbor);
                }
            }
        }
    }
    Ok(ret)
}

fn bound(posn: Vector3, world: &World) -> Vector3 {
    let dim = world.blocks().dim;
    Vector3::new(
        posn.x().min(dim.x() - 1).max(0),
        posn.y().min(dim.y() - 1).max(0),
        posn.z().min(dim.z() - 1).max(0),
    )
}

fn get_step(
    world: &World,
    g: &GameData,
    origin: Vector3,
    dir: Vector3,
    num_units: isize,
) -> Result<Vector3> {
    let max = bound((dir * num_units) + origin, world);
    let dir = dir * num_units.signum();
    let mut curr = origin;
    while curr != max {
        curr = curr + dir;
        let block = world.get_block_at(g, curr)?;
        if block.solid {
            curr = curr - dir;
            break;
        } else if let Ok(mob) = world.get_mobtemplate_at(curr, g) {
            if thread_rng().gen::<f64>() < mob.stats.get("agression", g)? {
                break;
            }
        }
    }
    Ok(curr)
}

const MAX_MOVE_SPEED: i64 = 10;
fn step(mut data: ActionData) -> Result<()> {
    let battle_map = data
        .battle_map
        .read()
        .map_err(|_| anyhow!("couldn't lock battle map"))?;
    if battle_map.get_opponent(ID::player(data.player_id)).is_ok() {
        return Err(anyhow!(
            "you can't move while fighting something, try using \"run\""
        ));
    }
    let player = get_mut(data.players, data.player_id)?
        .as_mut()
        .ok_or(anyhow!("invalid player id"))?;

    let move_speed = (player.stats().get("speed", data.g)?.round() as i64).min(MAX_MOVE_SPEED);
    let m = match (data.params.pop_front(), data.params.pop_front()) {
        (Some(Literal::String(s)), None) => match s.as_str() {
            "w" => Ok(Move::Y(-1)),
            "a" => Ok(Move::X(-1)),
            "s" => Ok(Move::Y(1)),
            "d" => Ok(Move::X(1)),
            "ww" => Ok(Move::Y(-move_speed)),
            "aa" => Ok(Move::X(-move_speed)),
            "ss" => Ok(Move::Y(move_speed)),
            "dd" => Ok(Move::X(move_speed)),
            "q" => Ok(Move::Z(false)),
            "e" => Ok(Move::Z(true)),
            _ => Err(anyhow!(BAD_ARGS)),
        },
        (Some(Literal::String(s)), Some(Literal::Number(Number::Int(i)))) => {
            if i < 0 {
                Err(anyhow!("you cannot travel a negative distance"))
            } else if i > move_speed {
                Err(anyhow!(format!(
                    "the maximum number of units you can travel is {}",
                    move_speed
                )))
            } else {
                match s.as_str() {
                    "w" => Ok(Move::Y(-i)),
                    "a" => Ok(Move::X(-i)),
                    "s" => Ok(Move::Y(i)),
                    "d" => Ok(Move::X(i)),
                    _ => Err(anyhow!(BAD_ARGS)),
                }
            }
        }
        _ => Err(anyhow!(BAD_ARGS)),
    }?;

    let world = data
        .world
        .read()
        .map_err(|_| anyhow!("couldn't lock world"))?;

    let new_posn = {
        let curr = *player.loc();
        match m {
            Move::X(num_units) => {
                let dir = Vector3::new(1, 0, 0);
                let num_units = num_units as isize;
                get_step(&world, data.g, curr, dir, num_units)
            }
            Move::Y(num_units) => {
                let dir = Vector3::new(0, 1, 0);
                let num_units = num_units as isize;
                get_step(&world, data.g, curr, dir, num_units)
            }
            Move::Z(down) => {
                if down {
                    if curr.z() == world.blocks().dim.z() - 1 {
                        Err(anyhow!("cannot move down, already at bottom layer"))
                    } else {
                        let block_filter = |x: &Block, _: Option<&MobTemplate>| x.z_passable;
                        let closest_z_passable =
                            find_closest_posn(&world, data.g, curr, &block_filter, 1)?;

                        let below = closest_z_passable + Vector3::new(0, 0, 1);
                        let block_filter = |x: &Block, _: Option<&MobTemplate>| !x.solid;
                        find_closest_posn(&world, data.g, below, &block_filter, u64::MAX)
                    }
                } else {
                    if curr.z() == 0 {
                        Err(anyhow!("cannot move up, already at top layer"))
                    } else {
                        let above = curr - Vector3::new(0, 0, 1);
                        if !(world
                            .get_block(data.g, world.blocks().index(above)?)?
                            .z_passable)
                        {
                            Err(anyhow!(
                                "you cannot ascend, there are solid blocks above you"
                            ))
                        } else {
                            let block_filter = |x: &Block, _: Option<&MobTemplate>| !x.solid;
                            find_closest_posn(&world, data.g, above, &block_filter, u64::MAX)
                        }
                    }
                }
            }
        }?
    };

    player.return_posn = player.loc().clone();
    player.loc_mut().set(new_posn);
    player.send_text(format!("moved to: {:?}\n", player.loc()));
    Ok(())
}

fn map(data: ActionData) -> Result<()> {
    let world = data
        .world
        .read()
        .map_err(|_| anyhow!("couldn't lock world"))?;

    let player = get_mut(data.players, data.player_id)?
        .as_ref()
        .ok_or(anyhow!("invalid player id"))?;

    let bounds = Bounds::get_bounds(
        &world,
        Vector3::new(0, 0, player.loc().z()),
        world.blocks().dim.x() as usize,
        world.blocks().dim.y() as usize,
    );
    let max_map_size = 30;
    let min_resolution = 2;
    let resolution = min_resolution.max((world.blocks().dim.x() as usize) / (max_map_size));

    let mut all = vec![];
    for p in data.players.iter() {
        all.push(*p.read().map_err(players_op)?);
    }

    let img = Image::new(&world, &all, data.g, &bounds, resolution)?;
    player.send_display(img);
    Ok(())
}

const VIEW_DIST: usize = 5;
fn disp(data: ActionData) -> Result<()> {
    let mut world = data
        .world
        .read()
        .map_err(|_| anyhow!("couldn't lock world"))?;
    let posn = *(get(&players, data.player_id)?.loc());
    let bounds = Bounds::get_bounds_centered(posn, VIEW_DIST, world.blocks().dim);
    let img = Image::new(&mut world, &players, data.g, &bounds, 1)?;
    let player = get_mut(&mut players, data.player_id)?;
    player.send_display(img);
    Ok(())
}

fn inv(data: ActionData) -> Result<()> {
    let mut players = data
        .players
        .write()
        .map_err(|_| anyhow!("couldn't lock players"))?;
    let player = get_mut(&mut players, data.player_id)?;
    player.send_text("here's what's in your inventory:\n".into());
    let inv = player.inventory().to_string();
    player.send_text(format!("{}\n", inv));
    Ok(())
}

fn equip(mut data: ActionData) -> Result<()> {
    let mut players = data
        .players
        .write()
        .map_err(|_| anyhow!("couldn't lock players"))?;
    let player = get_mut(&mut players, data.player_id)?;
    match (data.params.pop_front(), data.params.pop_front()) {
        (Some(Literal::String(s)), None) => match s.as_str() {
            "dequip" => {
                player.send_text("dequipping item...\n".into());
                player.dequip();
            }
            "equipped" => {}
            _ => return Err(anyhow!(BAD_ARGS)),
        },
        (Some(Literal::String(s1)), Some(Literal::String(s2))) => match s1.as_str() {
            "equip" => {
                let item_name = ItemName::checked_from(s2, data.g)?;
                player.equip(&item_name, data.g)?;
            }
            _ => return Err(anyhow!(BAD_ARGS)),
        },
        _ => return Err(anyhow!(BAD_ARGS)),
    };
    player.send_text(format!(
        "you currently have {} equipped\n",
        player.equipped().to_string()
    ));
    Ok(())
}

fn wear(mut data: ActionData) -> Result<()> {
    let mut players = data
        .players
        .write()
        .map_err(|_| anyhow!("couldn't lock players"))?;
    let player = get_mut(&mut players, data.player_id)?;
    match (data.params.pop_front(), data.params.pop_front()) {
        (Some(Literal::String(s)), None) => match s.as_str() {
            "unwear" => {
                player.unwear_all(data.g)?;
            }
            "wearing" => {}
            _ => return Err(anyhow!(BAD_ARGS)),
        },
        (Some(Literal::String(s1)), Some(Literal::String(s2))) => {
            let item_name = ItemName::checked_from(s2, data.g)?;
            match s1.as_str() {
                "wear" => {
                    player.wear(&item_name, data.g)?;
                }
                "unwear" => {
                    player.unwear(&item_name, data.g)?;
                }
                _ => return Err(anyhow!(BAD_ARGS)),
            }
        }
        _ => return Err(anyhow!(BAD_ARGS)),
    };
    player.send_text(format!(
        "you are currently wearing:\n {}\n",
        player.worn().to_string()
    ));
    Ok(())
}

fn upgrade(mut data: ActionData) -> Result<()> {
    let mut players = data
        .players
        .write()
        .map_err(|_| anyhow!("couldn't lock players"))?;

    data.params.pop_front(); // ignore the first parameter

    let stat;
    match data.params.pop_front() {
        Some(Literal::String(s)) => stat = s,
        _ => return Err(anyhow!(BAD_ARGS)),
    }
    let player = get_mut(&mut players, data.player_id)?;
    let val = player.stats().get_upgrade(stat.clone(), data.g)? as i64;
    if player.xp() < val * 100 {
        return Err(anyhow!(
            "you need {} xp to upgrade this stat, you have {} xp",
            val * 100,
            player.xp()
        ));
    } else {
        player.set_xp(player.xp() - val * 100);
    }
    player.stats_mut().upgrade(stat, data.g)?;
    Ok(())
}

const BATTLE_DIST: f64 = 10.0;
fn battle(data: ActionData) -> Result<()> {
    let mut battle_map = data
        .battle_map
        .write()
        .map_err(|_| anyhow!("couldn't lock battle map"))?;
    let mut players = data
        .players
        .write()
        .map_err(|_| anyhow!("couldn't lock players"))?;

    if battle_map.get_opponent(ID::player(data.player_id)).is_ok() {
        return Err(anyhow!("you are already fighting something"));
    }

    let battle_dist_sqr = BATTLE_DIST * BATTLE_DIST;
    let player_loc = get(&players, data.player_id)?.loc().clone();
    let mut opponent_id = None;
    for i in 0..players.len() {
        let player = &mut players[i];
        if i == data.player_id {
            continue;
        }
        if let Some(player) = player.as_mut() {
            if (player.loc().clone() - player_loc).sqr_mag() < battle_dist_sqr
                && battle_map.get_opponent(ID::player(i)).is_err()
            {
                opponent_id = Some(i);
                break;
            }
        }
    }

    if let Some(opp) = opponent_id {
        let (player, opponent) = get_two_mut(data.player_id, opp, &mut players)?;
        battle_map.init_battle(Box::new(player), Box::new(opponent), data.g)?;
        Ok(())
    } else {
        Err(anyhow!("no player in range"))
    }
}

fn run(data: ActionData) -> Result<()> {
    let mut players = data
        .players
        .write()
        .map_err(|_| anyhow!("couldn't lock players"))?;
    let player = get_mut(&mut players, data.player_id)?;
    let num_honour = player
        .inventory()
        .get(&ItemName::checked_from("honour".into(), data.g)?);

    if num_honour >= 20 {
        return Err(anyhow!("You're too honourable to run away sirrr"));
    }
    let posn = player.return_posn.clone();
    player.loc_mut().set(posn);

    let mut battle_map = data
        .battle_map
        .write()
        .map_err(|_| anyhow!("couldn't lock battle map"))?;

    if let Ok(opponent) = battle_map.get_opponent(ID::player(data.player_id)) {
        let mut world = data
            .world
            .write()
            .map_err(|_| anyhow!("couldn't lock world"))?;
        let (player, opp) =
            get_player_and_opponent(opponent, &mut players, data.player_id, &mut world)?;

        if let Ok(x) = opp.run() {
            player.send_text(format!("{}: \"{}\"\n", opp.name(), x));
            player.send_image("none".into());
        }
    }

    battle_map
        .end_battle(ID::player(data.player_id))
        .map_err(|_| anyhow!("you aren't fighting anything"))
}

fn pass(data: ActionData) -> Result<()> {
    let mut battle_map = data
        .battle_map
        .write()
        .map_err(|_| anyhow!("couldn't lock battle map"))?;
    if let Ok(opponent) = battle_map.get_opponent(ID::player(data.player_id)) {
        let mut players = data
            .players
            .write()
            .map_err(|_| anyhow!("couldn't lock players"))?;
        let mut world = data
            .world
            .write()
            .map_err(|_| anyhow!("couldn't lock world"))?;
        let (player, opp) =
            get_player_and_opponent(opponent, &mut players, data.player_id, &mut world)?;

        battle_map.do_turn(Box::new(player), opp, data.g)
    } else {
        Err(anyhow!("you aren't fighting anything"))
    }
}

fn account(mut data: ActionData) -> Result<()> {
    data.params.pop_front(); // ignore first argument

    let flag;
    let name;
    match (data.params.pop_front(), data.params.pop_front()) {
        (Some(Literal::String(f)), Some(Literal::String(n))) => {
            flag = f;
            name = n;
        }
        _ => return Err(anyhow!(BAD_ARGS)),
    }
    let mut players = data
        .players
        .write()
        .map_err(|_| anyhow!("couldn't lock players"))?;

    let player = get_mut(&mut players, data.player_id)?;

    let save_file = format!("{}/{}", PLAYER_SAVE_FOLDER, name);
    match flag.as_str() {
        "create" => {
            if Path::new(&save_file).exists() {
                return Err(anyhow!("A save file already exists with that username!"));
            }
            let mut f = File::create(save_file)?;
            f.write_all(&player.save()?.as_bytes())?;
            player.send_text(format!("created account '{}'\n", name));
        }
        "delete" => {
            if !Path::new(&save_file).exists() {
                return Err(anyhow!("No save file exists with that username!"));
            }
            fs::remove_file(&save_file)?;
            player.send_text(format!("deleted account '{}'\n", name));
        }
        "login" => {
            if player.username.is_some() {
                return Err(anyhow!("you are already logged in!"));
            }

            for player in players.iter() {
                if let Some(player) = player {
                    if let Some(username) = &player.username {
                        if username == &name {
                            return Err(anyhow!(
                                "someone with that username is already logged in!"
                            ));
                        }
                    }
                }
            }

            let player = get_mut(&mut players, data.player_id)?;

            let err = "No save file exists with that username!";
            player.load(fs::read_to_string(save_file).map_err(|_| anyhow!(err))?)?;
            player.username = Some(name);
            player.send_text(format!("you are now logged in as '{}'\n", player.name()));
        }
        _ => {
            return Err(anyhow!(
                "expected first arg to be either login, create, or delete"
            ))
        }
    };
    Ok(())
}

const MAX_SCAN_DIST: u64 = 5;
fn scan(data: ActionData) -> Result<()> {
    let start = {
        let players = data
            .players
            .read()
            .map_err(|_| anyhow!("couldn't lock players"))?;

        *get(&players, data.player_id)?.loc()
    };

    let world = data
        .world
        .read()
        .map_err(|_| anyhow!("couldn't lock world"))?;

    let filter = |_: &Block, mob: Option<&MobTemplate>| mob.is_some();

    let mut posns = find_all_posn(&world, data.g, start, &filter, MAX_SCAN_DIST, usize::MAX)?;

    if posns.len() == 0 {
        return Err(anyhow!("no mobs in scan range!"));
    }

    posns.retain(|&x| x != Vector3::zero());

    let mut players = data
        .players
        .write()
        .map_err(|_| anyhow!("couldn't lock players"))?;

    let player = get_mut(&mut players, data.player_id)?;
    for posn in posns {
        let mob_template = world.get_mobtemplate_at(posn, data.g)?;
        let diff = posn - start;
        let name = if mob_template.scan != "" {
            &mob_template.scan
        } else {
            &mob_template.name.0
        };
        let x = if diff.x() < 0 {
            format!("{} left", -diff.x())
        } else if diff.x() > 0 {
            format!("{} right", diff.x())
        } else {
            "".into()
        };
        let y = if diff.y() < 0 {
            format!(", {} up", -diff.x())
        } else if diff.x() > 0 {
            format!(", {} down", diff.x())
        } else {
            "".into()
        };
        player.send_text(format!("'{}' at {}{}\n", name, x, y));
    }
    Ok(())
}

fn info(mut data: ActionData) -> Result<()> {
    data.params.pop_front(); // ignore first argument

    let type_;
    let name;
    match (data.params.pop_front(), data.params.pop_front()) {
        (Some(Literal::String(t)), Some(Literal::String(n))) => {
            type_ = t;
            name = n;
        }
        _ => return Err(anyhow!(BAD_ARGS)),
    }

    let mut players = data
        .players
        .write()
        .map_err(|_| anyhow!("couldn't lock players"))?;

    let player = get_mut(&mut players, data.player_id)?;

    match type_.as_str() {
        "block" => {
            let b_name = BlockName::checked_from(name, data.g)?;
            let block = data
                .g
                .blocks
                .name_to_item
                .get(&b_name)
                .expect("blockname was validated");

            player.send_text(format!("{:#?}", block));
        }
        "mob" => {
            let m_name = MobName::checked_from(name, data.g)?;
            let mob_template = data
                .g
                .mob_templates
                .name_to_item
                .get(&m_name)
                .expect("mobname was validated");

            player.send_text(format!("{:#?}", mob_template));
        }
        "item" => {
            let item_name = ItemName::checked_from(name, data.g)?;
            let item = data
                .g
                .items
                .get(&item_name)
                .expect("itemname was validated");

            player.send_text(format!("{:#?}", item));
        }
        "opp" => {}
        _ => {
            return Err(anyhow!(
                "expected first arg to be either block, mob, or item!"
            ))
        }
    }

    Ok(())
}
