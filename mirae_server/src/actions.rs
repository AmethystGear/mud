use crate::{
    combat::{BattleMap, EntityType, ID},
    display::{Bounds, Image},
    entity::{Entity, MAX_NUM_EAT},
    gamedata::{
        block::Block,
        gamedata::{GameData, ItemName},
    },
    player::Player,
    vector3::Vector3,
    world::World,
};
use anyhow::{anyhow, Result};
use rand::Rng;
use serde_jacl::structs::{Literal, Number};
use std::collections::{HashSet, VecDeque};

const BAD_ARGS: &str =
    "bad arguments to command, try help <command> to see how to use it correctly";

pub fn dispatch(data: ActionData) -> Result<()> {
    match data.params.get(0) {
        Some(Literal::String(s)) => match s.as_str() {
            "disp" => disp(data),
            "inv" => inv(data),
            "eat" => eat(data),
            "w" | "a" | "s" | "d" | "ww" | "aa" | "ss" | "dd" | "q" | "e" => step(data),
            "wear" | "unwear" => wear(data),
            "equip" | "dequip" => equip(data),
            "use" => use_ability(data),
            "upgrade" => upgrade(data),
            "battle" => battle(data),
            "map" => map(data),
            "pass" => pass(data),
            "run" => run(data),
            _ => Err(anyhow!("invalid command")),
        },
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
    pub players: &'a mut Vec<Option<Player>>,
    pub world: &'a mut World,
    pub battle_map: &'a mut BattleMap,
    pub g: &'a GameData,
}

fn use_ability(mut data: ActionData) -> Result<()> {
    let player = get_mut(data.players, data.player_id)?;
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
    // CODE DUPLICATION WITH EAT, FIX LATER
    let ability = ability.clone();
    let player_id = ID::player(data.player_id);
    if let Ok(opponent) = data.battle_map.get_opponent(player_id) {
        if data.battle_map.turn(opponent)? {
            Err(anyhow!("it's not your turn!"))
        } else {
            let (player, opp) =
                get_player_and_opponent(opponent, data.players, data.player_id, data.world)?;

            player.run_ability(Some(opp), data.battle_map, ability, Some(item_name), data.g)?;

            let (player, opp) =
                get_player_and_opponent(opponent, data.players, data.player_id, data.world)?;

            data.battle_map.do_turn(Box::new(player), opp, data.g)
        }
    } else {
        player.run_ability(None, data.battle_map, ability, Some(item_name), data.g)
    }
}

fn eat(mut data: ActionData) -> Result<()> {
    let player = get_mut(data.players, data.player_id)?;
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

    let player_id = ID::player(data.player_id);
    if let Ok(opponent) = data.battle_map.get_opponent(player_id) {
        if data.battle_map.turn(opponent)? {
            Err(anyhow!("it's not your turn!"))
        } else {
            let (player, opp) =
                get_player_and_opponent(opponent, data.players, data.player_id, data.world)?;

            player.eat(Some(opp), data.battle_map, &item_name, amount, data.g)?;

            let (player, opp) =
                get_player_and_opponent(opponent, data.players, data.player_id, data.world)?;

            data.battle_map.do_turn(Box::new(player), opp, data.g)
        }
    } else {
        player.eat(None, data.battle_map, &item_name, amount, data.g)
    }
}

enum Move {
    X(i64),
    Y(i64),
    // true = down (greater z values), false = up (lesser z values)
    Z(bool),
}

fn find_closest_block(
    world: &World,
    g: &GameData,
    start: Vector3,
    block_filter: &dyn Fn(&Block) -> bool,
    max_dist: u64,
) -> Result<Vector3> {
    let mut visited = HashSet::new();
    let mut to_eval = VecDeque::new();
    to_eval.push_back((start, 0));
    visited.insert(start);
    while let Some((curr, dist)) = to_eval.pop_front() {
        let block = world.get_block(g, world.blocks().index(curr)?)?;
        if block_filter(block) {
            return Ok(curr);
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
    Err(anyhow!("no such block in range"))
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
    world: &mut World,
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
        } else if let Ok(mob) = world.get_mob_at_mut(curr, g) {
            if mob.rng().gen::<f64>() < mob.stats().get("agression", g)? {
                break;
            }
        }
    }
    Ok(curr)
}

const MAX_MOVE_SPEED: i64 = 10;
fn step(mut data: ActionData) -> Result<()> {
    if data
        .battle_map
        .get_opponent(ID::player(data.player_id))
        .is_ok()
    {
        return Err(anyhow!(
            "you can't move while fighting something, try using \"run\""
        ));
    }

    let player = get(data.players, data.player_id)?;
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

    let curr = *player.loc();
    let mut get_step = |dir, num_units| get_step(data.world, data.g, curr, dir, num_units);
    let new_posn = match m {
        Move::X(num_units) => get_step(Vector3::new(1, 0, 0), num_units as isize),
        Move::Y(num_units) => get_step(Vector3::new(0, 1, 0), num_units as isize),
        Move::Z(down) => {
            if down {
                if curr.z() == data.world.blocks().dim.z() - 1 {
                    Err(anyhow!("cannot move down, already at bottom layer"))
                } else {
                    let block_filter = |x: &Block| !x.z_passable;
                    let closest_z_passable =
                        find_closest_block(data.world, data.g, curr, &block_filter, 1)?;

                    let below = closest_z_passable + Vector3::new(0, 0, 1);
                    let block_filter = |x: &Block| !x.solid;
                    find_closest_block(data.world, data.g, below, &block_filter, u64::MAX)
                }
            } else {
                if curr.z() == 0 {
                    Err(anyhow!("cannot move up, already at top layer"))
                } else {
                    let above = curr - Vector3::new(0, 0, 1);
                    if !(data
                        .world
                        .get_block(data.g, data.world.blocks().index(above)?)?
                        .z_passable)
                    {
                        Err(anyhow!(
                            "you cannot ascend, there are solid blocks above you"
                        ))
                    } else {
                        let block_filter = |x: &Block| !x.solid;
                        find_closest_block(data.world, data.g, above, &block_filter, u64::MAX)
                    }
                }
            }
        }
    }?;
    let player = get_mut(data.players, data.player_id)?;
    player.return_posn = player.loc().clone();
    player.loc_mut().set(new_posn);
    player.send_text(format!("moved to: {:?}\n", player.loc()));
    if let Ok(mob) = data.world.get_mob_at_mut(player.loc().clone(), data.g) {
        data.battle_map.init_battle(Box::new(player), Box::new(mob), data.g)?;
        let mob_name = mob.name().unwrap();
        player.send_text(format!("{} is wearing {}\n", mob_name , mob.worn().to_string()));
        player.send_text(format!("{} has {} equipped.\n",  mob_name, mob.equipped().to_string()));
        player.send_text(format!("{}'s inventory is {}\n",  mob_name, mob.inventory().to_string()));
        player.send_text(format!("{}: {}\n", mob_name, mob.entrance().unwrap()));
        player.send_image(mob.display_img.clone());
    }
    Ok(())
}

fn map(data: ActionData) -> Result<()> {
    let posn = *(get(data.players, data.player_id)?.loc());
    let bounds = Bounds::get_bounds(
        data.world,
        Vector3::new(0, 0, posn.z()),
        data.world.blocks().dim.x() as usize,
        data.world.blocks().dim.y() as usize,
    );
    let max_map_size = 100;
    let min_resolution = 2;
    let resolution = min_resolution.max((data.world.blocks().dim.x() as usize) / (max_map_size));
    let img = Image::new(data.world, data.players, data.g, &bounds, resolution)?;
    let player = get_mut(data.players, data.player_id)?;
    player.send_display(img);
    Ok(())
}

const VIEW_DIST: usize = 5;
fn disp(data: ActionData) -> Result<()> {
    let posn = *(get(data.players, data.player_id)?.loc());
    let bounds = Bounds::get_bounds_centered(posn, VIEW_DIST, data.world.blocks().dim);
    let img = Image::new(data.world, data.players, data.g, &bounds, 1)?;
    let player = get_mut(data.players, data.player_id)?;
    player.send_display(img);
    Ok(())
}

fn inv(data: ActionData) -> Result<()> {
    let player = get_mut(data.players, data.player_id)?;
    player.send_text("here's what's in your inventory:\n".into());
    let inv = player.inventory().to_string();
    player.send_text(format!("{}\n", inv));
    Ok(())
}

fn equip(mut data: ActionData) -> Result<()> {
    let player = get_mut(data.players, data.player_id)?;
    match (data.params.pop_front(), data.params.pop_front()) {
        (Some(Literal::String(s)), None) => match s.as_str() {
            "dequip" => {
                player.send_text("dequipping item...\n".into());
                player.dequip()
            }
            "equip" => {}
            _ => return Err(anyhow!(BAD_ARGS)),
        },
        (Some(Literal::String(s1)), Some(Literal::String(s2))) => match s1.as_str() {
            "equip" => {
                let item_name = ItemName::checked_from(s2, data.g)?;
                player.equip(&item_name)?;
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
    let player = get_mut(data.players, data.player_id)?;
    match (data.params.pop_front(), data.params.pop_front()) {
        (Some(Literal::String(s)), None) => match s.as_str() {
            "unwear" => {
                player.unwear_all(data.g)?;
            }
            "wear" => {}
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
    data.params.pop_front(); // ignore the first parameter

    let stat;
    match data.params.pop_front() {
        Some(Literal::String(s)) => stat = s,
        _ => return Err(anyhow!(BAD_ARGS)),
    }
    let player = get_mut(data.players, data.player_id)?;
    let val = player.stats().get(stat.clone(), data.g)? as i64;
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
    if data
        .battle_map
        .get_opponent(ID::player(data.player_id))
        .is_ok()
    {
        return Err(anyhow!("you are already fighting something"));
    }

    let battle_dist_sqr = BATTLE_DIST * BATTLE_DIST;
    let player_loc = get(data.players, data.player_id)?.loc().clone();
    let mut opponent_id = None;
    for i in 0..data.players.len() {
        let player = &mut data.players[i];
        if i == data.player_id {
            continue;
        }
        if let Some(player) = player.as_mut() {
            if (player.loc().clone() - player_loc).sqr_mag() < battle_dist_sqr
                && data.battle_map.get_opponent(ID::player(i)).is_err()
            {
                opponent_id = Some(i);
                break;
            }
        }
    }

    if let Some(opp) = opponent_id {
        let (player, opponent) = get_two_mut(data.player_id, opp, data.players)?;
        data.battle_map
            .init_battle(Box::new(player), Box::new(opponent), data.g)?;

        Ok(())
    } else {
        Err(anyhow!("no player in range"))
    }
}

fn run(data: ActionData) -> Result<()> {
    let player = get_mut(data.players, data.player_id)?;
    let num_honour = player
        .inventory()
        .get(&ItemName::checked_from("honour".into(), data.g)?);

    if num_honour >= 20 {
        return Err(anyhow!("You're too honourable to run away sirrr"));
    }
    let posn = player.return_posn.clone();
    player.loc_mut().set(posn);

    if let Ok(opponent) = data.battle_map.get_opponent(ID::player(data.player_id)) {
        let (player, opp) =
            get_player_and_opponent(opponent, data.players, data.player_id, data.world)?;

        if let Some(x) = opp.run() {
            player.send_text(format!("{}: \"{}\"\n", opp.name().unwrap(), x));
            player.send_image("none".into());
        }
    }
    
    data.battle_map
        .end_battle(ID::player(data.player_id))
        .map_err(|_| anyhow!("you aren't fighting anything"))
}

fn pass(data : ActionData) -> Result<()> {
    if let Ok(opponent) = data.battle_map.get_opponent(ID::player(data.player_id)) {
        let (player, opp) =
            get_player_and_opponent(opponent, data.players, data.player_id, data.world)?;

        data.battle_map.do_turn(Box::new(player), opp, data.g)
    } else {
        Err(anyhow!("you aren't fighting anything"))
    }
}