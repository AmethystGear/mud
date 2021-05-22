use crate::{
    combat::{BattleMap, EntityType, ID},
    display::{Bounds, Image},
    entity::Entity,
    gamedata::{
        block::Block,
        gamedata::{BlockName, GameData, ItemName, MobName, Named},
        mobtemplate::MobTemplate,
    },
    player::Player,
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
    sync::{Arc, RwLock},
};
use websocket::websocket_base::header::names::ACCEPT;

const BAD_ARGS: &str = "bad arguments to command";

pub fn dispatch(data: ActionData) -> Result<()> {
    match data.params.get(0) {
        Some(Literal::String(s)) => {
            let func = match s.as_str() {
                "disp" => disp,
                "inv" | "inventory" => inv,
                "w" | "a" | "s" | "d" | "ww" | "aa" | "ss" | "dd" | "q" | "e" => step,
                "wear" | "unwear" | "wearing" => wear,
                "equip" | "dequip" | "equipped" => equip,
                "use" => use_ability,
                "do" => do_ability,
                "upgrade" => upgrade,
                "battle" => battle,
                "map" => map,
                "pass" => pass,
                "run" => run,
                "acc" | "account" => account,
                "scan" => scan,
                "info" => info,
                "descr" | "describe" => describe,
                "trade" => trade,
                "mine" => mine,
                _ => return Err(anyhow!("invalid command")),
            };
            func(data)
        }
        _ => Err(anyhow!("expected string as first parameter")),
    }
}

fn get_player_and_opponent<'a>(
    opponent: ID,
    players: &'a mut Vec<Option<Player>>,
    player_id: usize,
    world: &'a mut World,
) -> Result<(&'a mut Player, Box<&'a mut dyn Entity>)> {
    match opponent.enity_type {
        EntityType::Mob => {
            let player = get_mut(players, player_id)?;
            let opponent = world.get_mob_mut(opponent.id)?;
            Ok((player, Box::new(opponent as &mut dyn Entity)))
        }
        EntityType::Player => {
            let (player, opponent) = get_two_mut(player_id, opponent.id, players)?;
            Ok((player, Box::new(opponent as &mut dyn Entity)))
        }
    }
}

pub fn get_mut(players: &mut Vec<Option<Player>>, player_id: usize) -> Result<&mut Player> {
    players[player_id]
        .as_mut()
        .ok_or(anyhow!("invalid player id"))
}

pub fn get(players: &Vec<Option<Player>>, player_id: usize) -> Result<&Player> {
    players[player_id]
        .as_ref()
        .ok_or(anyhow!("invalid player id"))
}

pub fn get_two_mut(
    a: usize,
    b: usize,
    players: &mut Vec<Option<Player>>,
) -> Result<(&mut Player, &mut Player)> {
    let (head, tail) = players.split_at_mut(std::cmp::max(a, b) as usize);
    let err = "invalid player ids";
    if a > b {
        return Ok((
            tail[0].as_mut().ok_or(anyhow!(err.clone()))?,
            head[b as usize].as_mut().ok_or(anyhow!(err.clone()))?,
        ));
    } else if a < b {
        return Ok((
            head[a as usize].as_mut().ok_or(anyhow!(err.clone()))?,
            tail[0].as_mut().ok_or(anyhow!(err.clone()))?,
        ));
    } else {
        return Err(anyhow!("player ids cannot be equal"));
    }
}

pub fn get_two(a: usize, b: usize, players: &Vec<Option<Player>>) -> Result<(&Player, &Player)> {
    let (head, tail) = players.split_at(std::cmp::max(a, b) as usize);
    let err = "invalid player ids";
    if a > b {
        return Ok((
            tail[0].as_ref().ok_or(anyhow!(err.clone()))?,
            head[b as usize].as_ref().ok_or(anyhow!(err.clone()))?,
        ));
    } else if a < b {
        return Ok((
            head[a as usize].as_ref().ok_or(anyhow!(err.clone()))?,
            tail[0].as_ref().ok_or(anyhow!(err.clone()))?,
        ));
    } else {
        return Err(anyhow!("player ids cannot be equal"));
    }
}

pub struct ActionData<'a> {
    pub params: VecDeque<Literal>,
    pub player_id: usize,
    pub players: Arc<RwLock<Vec<Option<Player>>>>,
    pub world: Arc<RwLock<World>>,
    pub battle_map: Arc<RwLock<BattleMap>>,
    pub g: &'a GameData,
}

fn run_turn_with(
    battle_map: &mut BattleMap,
    player_id: ID,
    players: &mut Vec<Option<Player>>,
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
        let player = get_mut(players, player_id.id)?;
        func(player, None, g, battle_map)
    }
}

fn use_ability(mut data: ActionData) -> Result<()> {
    let mut players = data
        .players
        .write()
        .map_err(|_| anyhow!("couldn't lock players"))?;
    let player = get_mut(&mut players, data.player_id)?;
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

    let item = &data.g.items[item_name];
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
                mut opp: Option<Box<&mut dyn Entity>>,
                g: &GameData,
                battle_map: &mut BattleMap| {
        player.run_ability(
            &mut opp,
            battle_map,
            ability.clone(),
            &Some(item_name.clone()),
            g,
        )
    };

    run_turn_with(
        &mut battle_map,
        player_id,
        &mut players,
        &mut world,
        data.g,
        &func,
    )
}

fn do_ability(mut data: ActionData) -> Result<()> {
    let mut players = data
        .players
        .write()
        .map_err(|_| anyhow!("couldn't lock players"))?;
    let item_name;
    let amount;
    let ability;
    data.params.pop_front(); // ignore the first parameter
    match (
        data.params.pop_front(),
        data.params.pop_front(),
        data.params.pop_front(),
    ) {
        (
            Some(Literal::String(ability_name)),
            Some(Literal::String(item)),
            Some(Literal::Number(Number::Int(number))),
        ) => {
            item_name = ItemName::checked_from(item, data.g)?;
            let item = &data.g.items[&item_name];

            if let Some(a) = item.abilities.get(&ability_name) {
                ability = a.clone();
            } else {
                return Err(anyhow!(format!(
                    "the item {:?} doesn't have the ability {:?}. It has the abilities: {:?}",
                    item_name,
                    ability_name,
                    item.abilities.keys()
                )));
            }

            if !ability.run_without_equip {
                return Err(anyhow!("you must equip this item to run this ability!"));
            }

            if number > 0 && number as u64 <= ability.max_times_per_turn {
                amount = number as u64;
            } else if number <= 0 {
                return Err(anyhow!("can't run an ability on less than 1 item"));
            } else {
                return Err(anyhow!(
                    "you can only run this ability a maximum of {} times per turn",
                    ability.max_times_per_turn
                ));
            }
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

    let func = |player: &mut Player,
                mut opp: Option<Box<&mut dyn Entity>>,
                g: &GameData,
                battle_map: &mut BattleMap|
     -> Result<()> {
        for _ in 0..amount {
            if let Err(e) = player.run_ability(
                &mut opp,
                battle_map,
                ability.clone(),
                &Some(item_name.clone()),
                g,
            ) {
                player.send_text(format!("ERROR: {:?}", e));
                break;
            }
        }
        Ok(())
    };

    run_turn_with(
        &mut battle_map,
        player_id,
        &mut players,
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
    let mut players = data
        .players
        .write()
        .map_err(|_| anyhow!("couldn't lock players"))?;
    let player = get(&players, data.player_id)?;
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
    let player = get_mut(&mut players, data.player_id)?;
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
    let mut players = data
        .players
        .write()
        .map_err(|_| anyhow!("couldn't lock players"))?;
    let posn = *(get(&players, data.player_id)?.loc());
    let bounds = Bounds::get_bounds(
        &world,
        Vector3::new(0, 0, posn.z()),
        world.blocks().dim.x() as usize,
        world.blocks().dim.y() as usize,
    );
    let max_map_size = 30;
    let min_resolution = 2;
    let resolution = min_resolution.max((world.blocks().dim.x() as usize) / (max_map_size));
    let img = Image::new(&world, &players, data.g, &bounds, resolution)?;
    let player = get_mut(&mut players, data.player_id)?;
    player.send_display(img, true);
    Ok(())
}

const VIEW_DIST: usize = 5;
fn disp(data: ActionData) -> Result<()> {
    let world = data
        .world
        .read()
        .map_err(|_| anyhow!("couldn't lock world"))?;
    let mut players = data
        .players
        .write()
        .map_err(|_| anyhow!("couldn't lock players"))?;
    let posn = *(get(&players, data.player_id)?.loc());
    let bounds = Bounds::get_bounds_centered(posn, VIEW_DIST, world.blocks().dim);
    let img = Image::new(&world, &players, data.g, &bounds, 1)?;
    let player = get_mut(&mut players, data.player_id)?;
    player.send_display(img, false);
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

fn do_turn_if_in_battle(data: ActionData) -> Result<()> {
    let mut players = data
        .players
        .write()
        .map_err(|_| anyhow!("couldn't lock world"))?;

    let player = get_mut(&mut players, data.player_id)?;

    let mut battle_map = data
        .battle_map
        .write()
        .map_err(|_| anyhow!("couldn't lock battle map"))?;

    let mut world = data
        .world
        .write()
        .map_err(|_| anyhow!("couldn't lock world"))?;

    let player_id = player.id();
    if let Ok(opponent) = battle_map.get_opponent(player_id) {
        let (player, opp) =
            get_player_and_opponent(opponent, &mut players, player_id.id, &mut world)?;
        battle_map.do_turn(Box::new(player), opp, data.g)?;
    }
    Ok(())
}

fn equip(mut data: ActionData) -> Result<()> {
    {
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
    }
    do_turn_if_in_battle(data)
}

fn wear(mut data: ActionData) -> Result<()> {
    {
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
    }
    do_turn_if_in_battle(data)
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
    player.send_text("upgraded stat\n".into());
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
        battle_map.init_battle(Box::new(player), Box::new(opponent), false, data.g)?;
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

        if opp.id().enity_type == EntityType::Player {
            return Err(anyhow!("you can't run away from players"));
        }

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
    {
        let battle_map = data
            .battle_map
            .write()
            .map_err(|_| anyhow!("couldn't lock battle map"))?;
        if let Err(_) = battle_map.get_opponent(ID::player(data.player_id)) {
            return Err(anyhow!("you aren't fighting anything"));
        }
    }
    do_turn_if_in_battle(data)
}

fn account(mut data: ActionData) -> Result<()> {
    data.params.pop_front(); // ignore first argument

    let mut players = data
        .players
        .write()
        .map_err(|_| anyhow!("couldn't lock players"))?;

    let player = get_mut(&mut players, data.player_id)?;

    let flag;
    let name;
    match (data.params.pop_front(), data.params.pop_front()) {
        (Some(Literal::String(f)), Some(Literal::String(n))) => {
            flag = f;
            name = n;
        }
        (None, None) => {
            let username;
            if let Some(uname) = &player.username {
                username = uname.clone();
            } else {
                return Err(anyhow!("you are not logged in, no account to access"));
            }

            player.send_text(format!("logged in as: {}\n", username));
            return Ok(());
        }
        _ => return Err(anyhow!(BAD_ARGS)),
    }

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

    let mut posns = find_all_posn(&world, data.g, start, &filter, 6, usize::MAX)?;

    posns.retain(|&x| x != start);

    if posns.len() == 0 {
        return Err(anyhow!("no mobs in scan range!"));
    }

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
            format!(" {} left", -diff.x())
        } else if diff.x() > 0 {
            format!(" {} right", diff.x())
        } else {
            "".into()
        };
        let y = if diff.y() < 0 {
            format!(" {} up", -diff.y())
        } else if diff.y() > 0 {
            format!(" {} down", diff.y())
        } else {
            "".into()
        };
        player.send_text(format!("'{}' at{}{}\n", name, x, y));
    }
    Ok(())
}

fn get_info(entity: Box<&mut dyn Entity>, g: &GameData) -> String {
    let equip = format!("equip: {}\n", entity.equipped().to_string());
    let wear = format!("wearing:\n{}\n", entity.worn().to_string());
    let inventory = format!("inventory:\n{}\n", entity.inventory().to_string());
    let stats = format!("stats:\n{}", entity.stats().to_string(g));
    format!("{}{}{}{}", equip, wear, inventory, stats)
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
        (Some(Literal::String(t)), None) => {
            type_ = t;
            name = "".into();
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
            let block = &data.g.blocks.name_to_item[&b_name];
            player.send_text(format!("{:#?}\n", block));
        }
        "mob" => {
            let m_name = MobName::checked_from(name, data.g)?;
            let mob_template = &data.g.mob_templates.name_to_item[&m_name];
            player.send_text(format!("{:#?}\n", mob_template));
        }
        "item" => {
            let item_name = ItemName::checked_from(name, data.g)?;
            let item = &data.g.items[&item_name];
            player.send_text(format!("{:#?}\n", item));
        }
        "opp" => {
            let battle_map = data
                .battle_map
                .read()
                .map_err(|_| anyhow!("couldn't lock battle map"))?;
            if let Ok(opponent) = battle_map.get_opponent(ID::player(data.player_id)) {
                let mut world = data
                    .world
                    .write()
                    .map_err(|_| anyhow!("couldn't lock world"))?;
                let (player, opp) =
                    get_player_and_opponent(opponent, &mut players, data.player_id, &mut world)?;

                let info = get_info(opp, data.g);
                player.send_text(info);
            } else {
                return Err(anyhow!("you're not fighting anything"));
            }
        }
        "self" => {
            let info = get_info(Box::new(player), data.g);
            player.send_text(info);
        }
        _ => {
            return Err(anyhow!(
                "expected first arg to be either block, mob, or item"
            ))
        }
    }
    Ok(())
}

fn describe(mut data: ActionData) -> Result<()> {
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
        "mob" => {
            let m_name = MobName::checked_from(name, data.g)?;
            let mob_template = &data.g.mob_templates.name_to_item[&m_name];
            player.send_text(format!("\"{}\"\n", mob_template.description));
        }
        "item" => {
            let item_name = ItemName::checked_from(name, data.g)?;
            let item = &data.g.items[&item_name];
            player.send_text(format!("\"{}\"\n", item.description));
        }
        _ => {
            return Err(anyhow!(
                "expected first arg to be either block, mob, or item"
            ))
        }
    }
    Ok(())
}

fn trade(mut data: ActionData) -> Result<()> {
    data.params.pop_front(); // ignore first arg

    let mut players = data
        .players
        .write()
        .map_err(|_| anyhow!("couldn't lock players"))?;

    let player = get_mut(&mut players, data.player_id)?;

    let world = data
        .world
        .read()
        .map_err(|_| anyhow!("couldn't lock world"))?;

    let mob_template = world.get_mobtemplate_at(*player.loc(), data.g)?;
    if mob_template.trades.len() == 0 {
        return Err(anyhow!("you cannot trade with this mob!"));
    }

    match (data.params.pop_front(), data.params.pop_front()) {
        (
            Some(Literal::Number(Number::Int(trade_no))),
            Some(Literal::Number(Number::Int(num_times))),
        ) => {
            // do the requested trade
            if trade_no < 0 {
                return Err(anyhow!("trade number cannot be negative"));
            }
            let trade_no = trade_no as usize;
            let n_trades = mob_template.trades.len();
            if trade_no >= n_trades {
                return Err(anyhow!(format!(
                    "trade number must be less than {}",
                    n_trades
                )));
            }
            if num_times <= 0 {
                return Err(anyhow!("num times to do trade must at least be one"));
            }
            let trade = &mob_template.trades[trade_no];
            let in_cnt = trade.in_cnt as i64 * num_times;
            let out_cnt = trade.out_cnt * num_times as u64;

            player
                .inventory_mut()
                .change(trade.in_item.clone(), -in_cnt)?;
            player.inventory_mut().add(trade.out_item.clone(), out_cnt);
        }
        (None, None) => {
            // display the trades
            let mut i = 0;
            for trade in &mob_template.trades {
                player.send_text(format!("{}: {:?}\n", i, trade));
                i += 1;
            }
        }
        _ => return Err(anyhow!(BAD_ARGS)),
    }

    Ok(())
}

fn mine(mut data: ActionData) -> Result<()> {
    data.params.pop_front(); // ignore first arg

    let direction;
    let num_units;
    match (data.params.pop_front(), data.params.pop_front()) {
        (Some(Literal::String(dir)), Some(Literal::Number(Number::Int(len)))) => {
            if len < 1 || len > 5 {
                return Err(anyhow!(
                    "cannot break less than 1 block or more than 5 blocks"
                ));
            }
            num_units = len as isize;
            match dir.as_str() {
                "w" => {
                    direction = Vector3::new(0, -1, 0);
                }
                "a" => {
                    direction = Vector3::new(-1, 0, 0);
                }
                "s" => {
                    direction = Vector3::new(0, 1, 0);
                }
                "d" => {
                    direction = Vector3::new(1, 0, 0);
                }
                _ => return Err(anyhow!(BAD_ARGS)),
            }
        }
        _ => return Err(anyhow!(BAD_ARGS)),
    }

    let mut players = data
        .players
        .write()
        .map_err(|_| anyhow!("couldn't lock players"))?;

    let player = get_mut(&mut players, data.player_id)?;

    let mut world = data
        .world
        .write()
        .map_err(|_| anyhow!("couldn't lock world"))?;

    let start = *player.loc();
    for i in 1..(num_units + 1) {
        let curr = (direction * i) + start;
        let block = world.get_block_at(data.g, curr)?;
        if let Some(break_into) = &block.break_into {
            let block_id = data.g.get_block_id_by_blockname(break_into)?;
            world.blocks_mut().set(curr, block_id)?;
            if let Some(drop) = &block.drop {
                player.inventory_mut().add(drop.clone(), 1);
                player.send_text(format!("+1 '{}'\n", drop.0))
            }
        } else {
            return Err(anyhow!("cannot break {:?} at {:?}", block.name, curr));
        }
    }
    Ok(())
}
