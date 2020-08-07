extern crate rstring_builder;

use crate::display;
use crate::entities;
use crate::entities::SpawnedEntities;
use crate::player;
use crate::player::Player;
use crate::playerout::PlayerOut;
use crate::scanner;
use crate::scanner::Param;
use crate::stats;
use crate::world;
use crate::world::World;
use char_stream::CharStream;
use rstring_builder::StringBuilder;
use std::collections::HashMap;
use std::error::Error;
use std::u8;

type Res = std::result::Result<PlayerOut, Box<dyn Error>>;

#[derive(Clone)]
pub enum ActionFunc {
    A(fn(&mut Vec<Option<Player>>, &mut World) -> Res),
    B(fn(String, &Vec<scanner::Param>, u8, &mut Vec<Option<Player>>, &mut World) -> Res),
    C(fn(&Vec<scanner::Param>, u8, &mut Vec<Option<Player>>, &mut World) -> Res),
    D(fn(u8, &mut Vec<Option<Player>>, &mut World) -> Res),
    E(fn(&ActionMap, &Vec<scanner::Param>) -> Res),
    F(fn(&mut SpawnedEntities, u8, &mut Vec<Option<Player>>, &mut World) -> Res),
    G(
        fn(
            &mut SpawnedEntities,
            &Vec<scanner::Param>,
            u8,
            &mut Vec<Option<Player>>,
            &mut World,
        ) -> Res,
    ),
    H(fn(&Vec<scanner::Param>, u8, &mut Vec<Option<Player>>) -> Res),
}

pub struct Action {
    name: String,
    description: String,
    usage: String,
    keywords: String,
    func: ActionFunc,
}

impl Action {
    pub fn run(
        &self,
        s: Option<&mut SpawnedEntities>,
        a_map: Option<&ActionMap>,
        keyword: Option<String>,
        params: Option<&Vec<scanner::Param>>,
        player_id: Option<u8>,
        players: Option<&mut Vec<Option<Player>>>,
        world: Option<&mut World>,
    ) -> Option<Res> {
        let result: Res = match self.func {
            ActionFunc::A(x) => x(players?, world?),
            ActionFunc::B(x) => x(keyword?, params?, player_id?, players?, world?),
            ActionFunc::C(x) => x(params?, player_id?, players?, world?),
            ActionFunc::D(x) => x(player_id?, players?, world?),
            ActionFunc::E(x) => x(a_map?, params?),
            ActionFunc::F(x) => x(s?, player_id?, players?, world?),
            ActionFunc::G(x) => x(s?, params?, player_id?, players?, world?),
            ActionFunc::H(x) => x(params?, player_id?, players?),
        };
        return Some(result);
    }

    pub fn new(func: ActionFunc) -> Self {
        Action {
            name: "".to_string(),
            description: "".to_string(),
            usage: "".to_string(),
            keywords: "".to_string(),
            func: func,
        }
    }
}

impl Clone for Action {
    fn clone(&self) -> Action {
        Action {
            name: self.name.clone(),
            description: self.description.clone(),
            usage: self.usage.clone(),
            keywords: self.keywords.clone(),
            func: self.func.clone(),
        }
    }
}

pub struct ActionMap {
    command_word_map: HashMap<String, Action>,
}

impl ActionMap {
    pub fn new() -> Self {
        ActionMap {
            command_word_map: HashMap::new(),
        }
    }
}

pub fn get_action_and_params(
    map: &ActionMap,
    s: String,
) -> Result<(String, Vec<Param>, Action), Box<dyn Error>> {
    let mut scan = scanner::from(CharStream::from_string(s));
    let first_word = scanner::next(&mut scan)?;
    let params = scanner::get_params(&mut scan);
    let action = map
        .command_word_map
        .get(&first_word)
        .ok_or("that command doesn't exist!")?;
    return Ok((first_word.clone(), params, action.clone()));
}

pub fn add_action(map: &mut ActionMap, s: String, a: Action) {
    map.command_word_map.insert(s, a);
}

pub fn get_action_map() -> ActionMap {
    let mut m = ActionMap::new();
    add_action(
        &mut m,
        "help".to_string(),
        Action {
            name: "help".to_string(),
            description: "the help menu\n".to_string(),
            usage: "help|help action|help stat|help <x>\n".to_string(),
            keywords: "help".to_string(),
            func: ActionFunc::E(help.clone()),
        },
    );
    let move_action = Action {
        name: "move".to_string(),
        description: "Allows you to move in the map.\n\
                      If you just use w/a/s/d, it will move you by your 'speed' stat in that direction.\n\
                      You can also do w/a/s/d <x> to specify the number of units.\n".to_string(),
        usage:"w/a/s/d <optional: number of units to move>".to_string(),
        keywords: "move, movement, walk, position".to_string(),
        func: ActionFunc::B(step.clone())
    };
    add_action(&mut m, "w".to_string(), move_action.clone());
    add_action(&mut m, "a".to_string(), move_action.clone());
    add_action(&mut m, "s".to_string(), move_action.clone());
    add_action(&mut m, "d".to_string(), move_action.clone());
    add_action(&mut m, "ww".to_string(), move_action.clone());
    add_action(&mut m, "aa".to_string(), move_action.clone());
    add_action(&mut m, "ss".to_string(), move_action.clone());
    add_action(&mut m, "dd".to_string(), move_action.clone());
    add_action(
        &mut m,
        "disp".to_string(),
        Action {
            name: "display".to_string(),
            description: "display your surroundings.\n".to_string(),
            usage: "disp".to_string(),
            keywords: "show, display, see, look".to_string(),
            func: ActionFunc::D(disp.clone()),
        },
    );
    add_action(
        &mut m,
        "map".to_string(),
        Action {
            name: "map".to_string(),
            description: "display the world map.\n".to_string(),
            usage: "map".to_string(),
            keywords: "map, world".to_string(),
            func: ActionFunc::A(map.clone()),
        },
    );
    add_action(&mut m,
    "attack".to_string(),
    Action {
        name: "attack".to_string(),
        description: "attack object you are currently interacting with.\n".to_string(),
        usage: "attack <optional: name of attack (must be one of the attacks in your currently equipped item)>\n".to_string(),
        keywords: "attack, kill, damage, dmg".to_string(),
        func: ActionFunc::G(attack.clone())
    });
    add_action(
        &mut m,
        "equip".to_string(),
        Action {
            name: "equip".to_string(),
            description: "equip an item in your inventory\n".to_string(),
            usage: "equip <item name>|equip\n".to_string(),
            keywords: "equip, item".to_string(),
            func: ActionFunc::C(equip.clone()),
        },
    );
    add_action(
        &mut m,
        "battle".to_string(),
        Action {
            name: "battle".to_string(),
            description: "battle the closest player in your view.\n".to_string(),
            usage: "battle\n".to_string(),
            keywords: "battle, fight, combat".to_string(),
            func: ActionFunc::D(battle.clone()),
        },
    );
    add_action(
        &mut m,
        "stat".to_string(),
        Action {
            name: "stat".to_string(),
            description: "display your stats, your opponent's stats, or the stats of an item.\n"
                .to_string(),
            usage: "stat|stat opponent|stat <item_name>\n".to_string(),
            keywords: "stats, info".to_string(),
            func: ActionFunc::G(stat.clone()),
        },
    );
    add_action(
        &mut m,
        "upgrade".to_string(),
        Action {
            name: "upgrade".to_string(),
            description: "upgrade a stat.\n".to_string(),
            usage: "upgrade <x>".to_string(),
            keywords: "upgrade, level".to_string(),
            func: ActionFunc::H(upgrade.clone()),
        },
    );
    add_action(
        &mut m,
        "eat".to_string(),
        Action {
            name: "eat".to_string(),
            description:
                "eat something in your inventory... I mean seriously, do you need an explanation??"
                    .to_string(),
            usage: "eat <x>".to_string(),
            keywords: "eat, consume".to_string(),
            func: ActionFunc::C(eat.clone()),
        },
    );
    add_action(
        &mut m,
        "wear".to_string(),
        Action {
            name: "wear".to_string(),
            description: "wear an item in your inventory, or unwear all items.".to_string(),
            usage: "wear <x>|wear".to_string(),
            keywords: "wear".to_string(),
            func: ActionFunc::G(wear.clone()),
        },
    );
    return m;
}

fn action_to_string(action: &Action) -> String {
    let mut out = StringBuilder::new();
    out.append(action.name.clone());
    out.append('\n');
    out.append("description:\n");
    out.append(action.description.clone());
    out.append("usage:\n");
    out.append(action.usage.clone());
    return out.string();
}

fn bind(a: i32, min: i32, max: i32) -> i32 {
    if min > max {
        panic!("min > max!");
    }
    if a < min {
        return min;
    } else if a > max {
        return max;
    } else {
        return a;
    }
}

fn displace(x_origin: i32, y_origin: i32, x_axis: bool, dist: i32) -> (i32, i32) {
    if x_axis {
        return (
            bind(x_origin + dist, 0, (world::MAP_SIZE - 1) as i32),
            y_origin,
        );
    } else {
        return (
            x_origin,
            bind(y_origin + dist, 0, (world::MAP_SIZE - 1) as i32),
        );
    }
}

fn get_step(
    x_origin: i32,
    y_origin: i32,
    x_axis: bool,
    num_units: i32,
    world: &World,
) -> Result<(u16, u16), Box<dyn Error>> {
    let max = displace(x_origin, y_origin, x_axis, num_units);
    let mut current = displace(x_origin, y_origin, x_axis, num_units.signum());
    let mut backtrack = false;
    while !(current.0 == max.0 && current.1 == max.1) {
        current = displace(current.0, current.1, x_axis, num_units.signum());
        let block = world::get_block(world, current.0 as u16, current.1 as u16)?;
        if stats::has_prop(block, "solid") {
            backtrack = true;
            break;
        }
        if world::has_entity(world, current.0 as u16, current.1 as u16) {
            let properties =
                world::get_entity_properties(world, current.0 as u16, current.1 as u16)
                    .ok_or("can't find entity properties")?;
            let mob_stats =
                stats::get_or_else(properties, "stats", &stats::Value::Box(stats::Stats::new()))
                    .as_box()?;
            let agression =
                stats::get_or_else(&mob_stats, "agression", &stats::Value::Float(0.0f64))
                    .as_flt()?;
            let v: f64 = rand::random::<f64>();
            if v < agression {
                break;
            }
        }
    }
    let block = world::get_block(world, current.0 as u16, current.1 as u16)?;
    if stats::has_prop(block, "solid") {
        backtrack = true;
    }
    let end;
    if backtrack {
        end = displace(current.0, current.1, x_axis, -num_units.signum());
    } else {
        end = current;
    }
    return Ok((end.0 as u16, end.1 as u16));
}

fn help(action_map: &ActionMap, params: &Vec<scanner::Param>) -> Res {
    let mut out = PlayerOut::new();
    if params.is_empty() {
        out.append("welcome to the help menu!\n");
        out.append("type 'help action' to list all the stuff you can do!\n");
        out.append("type 'help stat' to learn more about stats!\n");
        out.append(
            "type 'help <x>' and I'll try to search for an action that matches your query!\n",
        );
    } else if params.len() == 1 {
        if let Param::String(string) = &params[0] {
            if string == "action" {
                out.append("listing all the actions:\n");
                for action in action_map.command_word_map.values() {
                    out.append(action_to_string(action));
                }
            } else if string == "stat" {
                out.append("Base Stats vs. Stats:\n");
                out.append("base stats --> the maximum values that your stats can attain.\n");
                out.append("stats --> the actual current value of your stats.\n");
                out.append("for example, your health under 'stats' might be 7, but your health under 'base stats' might be 10.");
                out.append(" that means your current health is 7, and your max health is 10.\n");
                out.append("Stat Descriptions:\n");
                out.append(
                    "health --> health (duh). If this reaches 0, you die and you are respawned.\n",
                );
                out.append("speed --> determines how many units you can move per turn, and whether you go first/how many turns you or your opponent takes in a battle.\n");
                out.append("dmg --> the base damage you can deal per turn in a battle.\n");
                out.append("view --> the distance that you can see. If you increase view, your 'disp' command will show a larger area.\n");
            } else {
                let mut any_matches = false;
                for action in action_map.command_word_map.values() {
                    if action.description.contains(string.as_str())
                        || action.name.contains(string.as_str())
                        || action.keywords.contains(string.as_str())
                    {
                        out.append(action_to_string(action));
                        any_matches = true;
                    }
                }
                if !any_matches {
                    out.append("there were no matches for your query!\n");
                }
            }
        } else {
            return Err("was expecting first parameter to be a string".into());
        }
    } else {
        return Err("was expecting only one parameter".into());
    }
    return Ok(out);
}

fn step(
    keyword: String,
    params: &Vec<Param>,
    player_id: u8,
    players: &mut Vec<Option<Player>>,
    world: &mut World,
) -> Res {
    let mut player = players[player_id as usize]
        .as_mut()
        .ok_or("player id invalid")?;
    if player.opponent().is_some() {
        return Err("You can't just run away! Die with honour, scum!".into());
    }
    let num_units;
    if params.is_empty() {
        if keyword == "ww" || keyword == "aa" || keyword == "ss" || keyword == "dd" {
            num_units = player::get_stat(&player, "speed")?;
        } else if keyword == "w" || keyword == "a" || keyword == "s" || keyword == "d" {
            num_units = 1;
        } else {
            unreachable!("keyword must be one of the above move options!")
        }
    } else {
        let num = params[0].as_int();
        match num {
            Ok(n) => {
                if n > player::get_stat(&player, "speed")? {
                    return Err(format!("You can only move {} units per turn!", n).into());
                } else {
                    num_units = n;
                }
            }
            Err(_) => return Err("expected an integer!".into()),
        }
    }
    let num_units = std::cmp::min(num_units, player::MAX_PHYSICAL_SPEED);
    let new_posn;
    let mov = keyword.chars().next().expect("keyword can't be empty!");
    if mov == 'w' {
        new_posn = get_step(
            player::x(&player)? as i32,
            player::y(&player)? as i32,
            false,
            -num_units as i32,
            world,
        )?;
    } else if mov == 'a' {
        new_posn = get_step(
            player::x(&player)? as i32,
            player::y(&player)? as i32,
            true,
            -num_units as i32,
            world,
        )?;
    } else if mov == 's' {
        new_posn = get_step(
            player::x(&player)? as i32,
            player::y(&player)? as i32,
            false,
            num_units as i32,
            world,
        )?;
    } else if mov == 'd' {
        new_posn = get_step(
            player::x(&player)? as i32,
            player::y(&player)? as i32,
            true,
            num_units as i32,
            world,
        )?;
    } else {
        unreachable!("keyword must start with w, a, s, or d");
    }
    player::set_posn(&mut player, new_posn.0, new_posn.1)?;
    player.set_interact(false);
    player::reset_to_base(&mut player)?;
    return disp(player_id, players, world);
}

fn disp(player_id: u8, players: &mut Vec<Option<Player>>, world: &mut World) -> Res {
    let player = players[player_id as usize]
        .as_mut()
        .ok_or("player id is invalid!")?;
    let p_x = player::x(&player)?;
    let p_y = player::y(&player)?;
    let view = player::get_stat(&player, "view")? as u16;
    let mut out = PlayerOut::new();
    out.append(format!("{},{}\n", p_x, p_y));
    let x;
    if view > p_x {
        x = 0;
    } else {
        x = p_x - view;
    }
    let y;
    if view > p_y {
        y = 0;
    } else {
        y = p_y - view;
    }

    let view = std::cmp::min(view, player::MAX_PHYSICAL_VIEW as u16);

    let img = display::Img {
        x_origin: x,
        y_origin: y,
        x_length: std::cmp::min(2 * view + 1, world::MAP_SIZE - x),
        y_length: std::cmp::min(2 * view + 1, world::MAP_SIZE - y),
        resolution: 1,
    };
    out.append_img(world, players, img)?;
    return Ok(out);
}

fn map(players: &mut Vec<Option<Player>>, world: &mut World) -> Res {
    let mut out = PlayerOut::new();
    let img = display::Img {
        x_origin: 0,
        x_length: world::MAP_SIZE,
        y_origin: 0,
        y_length: world::MAP_SIZE,
        resolution: world::MAP_SIZE / 100,
    };
    out.append_img(world, players, img)?;
    return Ok(out);
}

pub fn get_two_players(
    a: u8,
    b: u8,
    players: &mut Vec<Option<Player>>,
) -> Option<(&mut Player, &mut Player)> {
    let (head, tail) = players.split_at_mut(std::cmp::max(a, b) as usize);
    if a > b {
        return Some((tail[0].as_mut()?, head[b as usize].as_mut()?));
    } else if a < b {
        return Some((head[a as usize].as_mut()?, tail[0].as_mut()?));
    } else {
        return None;
    }
}

pub fn attack(
    spawned_entities: &mut SpawnedEntities,
    params: &Vec<scanner::Param>,
    player_id: u8,
    players: &mut Vec<Option<Player>>,
    world: &mut World,
) -> Res {
    let opponent_id;
    {
        let player = players[player_id as usize]
            .as_ref()
            .ok_or("player id is invalid")?;
        opponent_id = player.opponent();
    }
    let opponent;
    let mut player;
    if let Some(opponent_id) = opponent_id {
        let (opp, p) = get_two_players(opponent_id, player_id, players)
            .ok_or("could not get multiple player refs!")?;
        opponent = Some(opp);
        player = p;
    } else {
        player = players[player_id as usize]
            .as_mut()
            .ok_or("player id is invalid")?;
        opponent = None;
    }
    let energy_cost;
    let physical_dmg;
    let magic_dmg;
    let at_name;
    if params.len() == 0 {
        energy_cost = 0;
        physical_dmg = player::get_stat(&player, "dmg")?;
        magic_dmg = 0;
        at_name = "base attack".to_string();
    } else {
        if let Param::String(attack_name) = &params[0] {
            let abilities = stats::get_or_else(
                player.equip().ok_or("you don't have anything equipped!")?,
                "abilities",
                &stats::Value::Box(stats::Stats::new()),
            )
            .as_box()?;
            if !stats::has_var(&abilities, attack_name) {
                return Err("your equipped weapon does not have that ability!".into());
            }
            let ability = stats::get(&abilities, attack_name)?.as_box()?;
            energy_cost =
                stats::get_or_else(&ability, "energy_cost", &stats::Value::Int(0)).as_int()?;
            physical_dmg =
                stats::get_or_else(&ability, "physical_dmg", &stats::Value::Int(0)).as_int()?;
            magic_dmg =
                stats::get_or_else(&ability, "magic_dmg", &stats::Value::Int(0)).as_int()?;
            at_name = attack_name.clone();
        } else {
            return Err("attack should be a string".into());
        }
    }
    if player::get_stat(&player, "energy")? >= energy_cost {
        player::change_stat(&mut player, "energy", -energy_cost)?;
        if player.opponent().is_none() {
            // fighting entity
            if !entities::has_entity(spawned_entities, player::x(player)?, player::y(player)?) {
                return Err("you aren't fighting anything!".into());
            }
            let damage_opponent = entities::get_entity_action(
                spawned_entities,
                "dmg".to_string(),
                player::x(player)?,
                player::y(player)?,
            );
            let damage_opponent = damage_opponent.ok_or("you cannot damage this entity!")?;
            let player;
            let res = damage_opponent.run(
                Some(spawned_entities),
                None,
                None,
                Some(&vec![Param::Int(physical_dmg), Param::Int(magic_dmg)]),
                Some(player_id),
                Some(players),
                Some(world),
            );

            let res = res.ok_or("bad None parameters to damage_opponent!")?;

            player = players[player_id as usize]
                .as_mut()
                .ok_or("player id invalid!")?;
            if !entities::has_entity(spawned_entities, player::x(player)?, player::y(player)?) {
                return res;
            }
            let mut out = res?;
            let entity_speed;
            {
                let entity = entities::get_entity_mut(
                    spawned_entities,
                    player::x(player)?,
                    player::y(player)?,
                )
                .ok_or("cannot retrieve entity")?;
                entity_speed =
                    stats::get_or_else(entity.mut_data(), "speed", &stats::Value::Int(0))
                        .as_int()?;
            }
            let mob_attack = entities::get_entity_action(
                spawned_entities,
                "attack".to_string(),
                player::x(player)?,
                player::y(player)?,
            );
            let player_speed = player::get_stat(&player, "speed")?;
            player.add_entity_cumulative_speed(entity_speed);
            if mob_attack.is_some() && player.entity_cumulative_speed() >= player_speed {
                player.zero_entity_cumulative_speed();
                let mob_attack = mob_attack.ok_or("cannot retrieve mob function")?;
                let res = mob_attack.run(
                    Some(spawned_entities),
                    None,
                    None,
                    None,
                    Some(player_id),
                    Some(players),
                    Some(world),
                );
                let res = res.ok_or("bad None params to mob_attack function")?;
                out.append_player_out(res?);
            }
            return Ok(out);
        } else {
            // fighting another player
            if !player::turn(&player) {
                return Err("It's not your turn sirrrrrrrr, just a minute sirrrrrrrrrrrrrrrrrrr.\n\
                            Be honourable and just wait for your opponent to finish attacking you sirrrrrrrr....".into());
            }
            let mut out = PlayerOut::new();
            let mut opponent = opponent.ok_or("invalid player id for opponent!")?;
            out.append(format!(
                "You used {}, dealing {} damage.\n",
                at_name,
                (physical_dmg + magic_dmg)
            ));
            player::send_str(
                &opponent,
                format!(
                    "Your opponent used {}, dealing {} damage.\n",
                    at_name,
                    (physical_dmg + magic_dmg)
                ),
            )?;
            player::change_stat(&mut opponent, "health", -(physical_dmg + magic_dmg))?;
            if player::is_dead(&opponent)? {
                out.append("Congrats for murdering your opponent!!!!\n");
                player::send_str(&opponent, format!("You were killed bigly.\n"))?;
                player.set_opponent(None);
                opponent.set_opponent(None);
                player::set_turn(&mut player, false);
                player::set_turn(&mut opponent, false);
                player::respawn(&mut opponent, world)?;
                return Ok(out);
            }
            opponent.add_entity_cumulative_speed(player::get_stat(&opponent, "speed")?);
            if player.entity_cumulative_speed() <= opponent.entity_cumulative_speed() {
                player::set_turn(&mut player, false);
                player::set_turn(&mut opponent, true);
                out.append("your turn has ended!\n");
                player::send_str(&opponent, "your turn has started!\n")?;
            } else {
                out.append("it is still your turn.\n");
                player::send_str(&opponent, "it is still your opponent's turn.\n")?;
            }
            return Ok(out);
        }
    } else {
        return Err("You don't have enough energy for that ability!".into());
    }
}

pub fn battle(player_id: u8, players: &mut Vec<Option<Player>>, _world: &mut World) -> Res {
    let p_x;
    let p_y;
    let view;
    {
        let player = players[player_id as usize]
            .as_ref()
            .ok_or("invalid player id")?;
        p_x = player::x(&player)?;
        p_y = player::y(&player)?;
        view = player::get_stat(&player, "view")? as usize;
    }

    let mut least_dist = std::usize::MAX;
    let mut opponent = None;
    for i in 0..players.len() {
        if players[i].is_none()
            || i == player_id as usize
            || players[i]
                .as_ref()
                .ok_or("invalid player id")?
                .opponent()
                .is_some()
        {
            continue;
        }
        let opp = players[i].as_ref().ok_or("invalid opponent player id")?;
        let dist_x = (player::x(&opp)? as i32 - p_x as i32) as isize;
        let dist_y = (player::y(&opp)? as i32 - p_y as i32) as isize;
        let dist = (dist_x * dist_x + dist_y * dist_y) as usize;
        if dist < least_dist && dist < (view * view * 2) {
            opponent = Some(i as u8);
            least_dist = dist;
        }
    }

    if let Some(opponent) = opponent {
        let (player, opp) =
            get_two_players(player_id, opponent, players).ok_or("could not get two player ids")?;

        let mut out = PlayerOut::new();
        player.set_opponent(Some(opponent));
        out.append(format!("You are attacking player {}\n", opponent));
        opp.set_opponent(Some(player_id));
        let player_speed = player::get_stat(&player, "speed")?;
        let opponent_speed = player::get_stat(&opp, "speed")?;
        player.zero_entity_cumulative_speed();
        opp.zero_entity_cumulative_speed();
        if player_speed >= opponent_speed {
            player::set_turn(player, true);
            player::set_turn(opp, false);
            player.add_entity_cumulative_speed(player_speed);
            out.append("It is your turn!\n");

            player::send_str(
                &opp,
                "Another player is battling you! It is their turn.\n".to_string(),
            )?;
        } else {
            player::set_turn(player, false);
            player::set_turn(opp, true);
            opp.add_entity_cumulative_speed(opponent_speed);
            out.append("It is your opponent's turn!\n");
            player::send_str(
                &opp,
                "Another player is battling you! It is your turn!\n".to_string(),
            )?;
        }
        return Ok(out);
    } else {
        let player = players[player_id as usize]
            .as_mut()
            .ok_or("invalid player id")?;
        player.set_opponent(None);
        return Err("no availible players in range!".into());
    }
}

pub fn equip(
    params: &Vec<scanner::Param>,
    player_id: u8,
    players: &mut Vec<Option<Player>>,
    world: &mut World,
) -> Res {
    let mut out = PlayerOut::new();
    let player = players[player_id as usize]
        .as_mut()
        .ok_or("invalid player id")?;
    if params.is_empty() {
        out.append("dequipping item.\n");
        player.set_equip(None);
    } else if params.len() == 1 {
        let inventory = stats::get(player.data(), "inventory")?.as_box()?;
        let items = stats::get_var_names(&inventory);
        let selected_item = params[0].as_string();
        if selected_item.is_err() {
            return Err("expected string as first parameter!".into());
        }
        let selected_item = selected_item?;
        if items.contains(&selected_item) {
            out.append("equipping item.\n");
            let it = &world.items();
            let item = stats::get(it, selected_item.as_str());
            if item.is_err() {
                return Err("That item is not defined, and cannot be equipped!".into());
            }
            player.set_equip(Some(item?.as_box()?));
        } else {
            return Err("You don't have that item!".into());
        }
    } else {
        return Err("You can only equip one item!".into());
    }
    return Ok(out);
}

fn stat(
    entities: &mut SpawnedEntities,
    params: &Vec<scanner::Param>,
    player_id: u8,
    players: &mut Vec<Option<Player>>,
    world: &mut World,
) -> Res {
    let mut out = PlayerOut::new();
    let player = players[player_id as usize]
        .as_mut()
        .ok_or("invalid player id!")?;
    if params.is_empty() {
        out.append(stats::string(player.data())?);
    } else {
        let val = params[0].as_string()?;
        if val == "opponent" {
            if let Some(opponent) = player.opponent() {
                let (opponent, _) = get_two_players(opponent, player_id, players)
                    .ok_or("could not get two players")?;
                out.append(stats::string(opponent.data())?);
            } else {
                let entity =
                    entities::get_entity_mut(entities, player::x(player)?, player::y(player)?);
                if entity.is_none() {
                    out.append("no opponent!\n");
                } else {
                    let entity = entity.ok_or("could not get entity")?;
                    out.append("mutable data:\n");
                    out.append(stats::string(&entity.mut_data().clone())?);
                    out.append("regular data:\n");
                    out.append(stats::string(&entity.data())?);
                }
            }
        } else {
            if stats::has_var(&world.items(), &val) {
                out.append(stats::string(&stats::get(&world.items(), &val)?.as_box()?)?);
            } else {
                return Err("that item doesn't exist!".into());
            }
        }
    }
    return Ok(out);
}

fn eat(
    params: &Vec<scanner::Param>,
    player_id: u8,
    players: &mut Vec<Option<Player>>,
    world: &mut World,
) -> Res {
    let player = players[player_id as usize]
        .as_mut()
        .ok_or("invalid player id")?;
    let inv = player::get_inventory(player)?;
    let msg = "expected exactly one item!";
    if params.len() != 1 {
        return Err(msg.into());
    }
    let item = params[0].as_string();
    if item.is_err() {
        return Err(msg.into());
    }
    let item = item?;
    if stats::has_var(&inv, item.as_str()) {
        player::remove_item_from_inventory(player, item.as_str())?;
        let item = stats::get_or_else(
            &world.items(),
            item.as_str(),
            &stats::Value::Box(stats::Stats::new()),
        )
        .as_box()?;
        let health_gain =
            stats::get_or_else(&item, "health_gain", &stats::Value::Int(0)).as_int()?;
        let energy_gain =
            stats::get_or_else(&item, "energy_gain", &stats::Value::Int(0)).as_int()?;
        player::change_stat(player, "health", health_gain)?;
        player::change_stat(player, "energy", energy_gain)?;
        let mut out = PlayerOut::new();
        out.append(format!(
            "you got {} health, and {} energy\n",
            health_gain, energy_gain
        ));
        return Ok(out);
    } else {
        return Err("you don't have that item!".into());
    }
}

fn upgrade(params: &Vec<scanner::Param>, player_id: u8, players: &mut Vec<Option<Player>>) -> Res {
    let mut player = players[player_id as usize]
        .as_mut()
        .ok_or("invalid player id!")?;
    if params.len() != 1 {
        return Err("expected exactly one stat to upgrade!".into());
    }
    if let scanner::Param::String(s) = &params[0] {
        player::upgrade_stat(&mut player, s.as_str())?;
        player::reset_to_base(&mut player)?;
        let mut out = PlayerOut::new();
        out.append("upgraded stat.\n");
        return Ok(out);
    } else {
        return Err("expected only one parameter, and expected it to be a stat name!".into());
    }
}

fn wear(
    spawned_entities: &mut SpawnedEntities,
    params: &Vec<scanner::Param>,
    player_id: u8,
    players: &mut Vec<Option<Player>>,
    world: &mut World,
) -> Res {
    let mut player = players[player_id as usize]
        .as_mut()
        .ok_or("invalid player id!")?;
    if player.opponent().is_some()
        || entities::has_entity(spawned_entities, player::x(player)?, player::y(player)?)
    {
        return Err("you cannot wear items while fighting!".into());
    }
    if params.len() > 1 {
        return Err("expected exactly one item to wear!".into());
    } else if params.len() == 0 {
        player::unwear_all(player)?;
        let mut out = PlayerOut::new();
        out.append("unwore all items!\n");
        return Ok(out);
    } else {
        if let scanner::Param::String(s) = &params[0] {
            if !stats::has_var(&player::get_inventory(&player)?, s.as_str()) {
                return Err("You do not have that item in your inventory!".into());
            }
            let it = &world.items();
            let item = stats::get(it, s);
            if item.is_err() {
                return Err("That item is not defined, and cannot be worn!".into());
            }
            let item = item?.as_box()?;
            if !stats::has_var(&item, "buffs") {
                return Err("You cannot wear this item!".into());
            }
            player::wear(
                &mut player,
                s.to_string(),
                stats::get(&item, "buffs")?.as_box()?,
            )?;
            player::reset_to_base(&mut player)?;
            let mut out = PlayerOut::new();
            out.append("wore item.\n");
            return Ok(out);
        } else {
            return Err("expected only one parameter, and expected it to be an item name!".into());
        }
    }
}
