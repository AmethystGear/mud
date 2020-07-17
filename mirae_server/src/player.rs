extern crate char_stream;
extern crate rand;

use crate::world::World;
use crate::world;
use std::sync::mpsc::Sender;
use crate::stats;
use crate::scanner;
use crate::stats::Value;
use crate::stats::Stats;
use char_stream::CharStream;
use std::fs::File;
use rand::Rng;

const DEFAULT : &str = "config/player_defaults.txt";
const PLAYER_DATA : &str = "save/player_data.txt";

pub struct Player {
    data : Stats,
    opponent : Option<u8>,
    equip : Option<Stats>,
    sender : Sender<(String, Option<u8>)>,
    cumulative_speed : i64,
    interact : bool,
    turn : bool
}

impl Player {
    pub fn data(&self) -> &Stats {
        return &(self.data);
    }

    pub fn opponent(&self) -> Option<u8> {
        return self.opponent.clone();
    }

    pub fn set_opponent(&mut self, opponent : Option<u8>) {
        self.opponent = opponent;
    }

    pub fn equip(&self) -> Option<&Stats> {
        if self.equip.is_none() {
            return None;
        } else {
            return Some(self.equip.as_ref().unwrap());
        }
    }

    pub fn set_equip(&mut self, item : Option<Stats>) {
        self.equip = item;
    }

    pub fn set_interact(&mut self, interact : bool) {
        self.interact = interact;
    }

    pub fn interact(&self) -> bool {
        return self.interact;
    }

    pub fn zero_entity_cumulative_speed(&mut self) {
        self.cumulative_speed = 0;
    }

    pub fn entity_cumulative_speed(&self) -> i64 {
        return self.cumulative_speed;
    }

    pub fn add_entity_cumulative_speed(&mut self, add : i64) {
        self.cumulative_speed += add;
    }
}

pub fn from(x : u16, y : u16, id : u8, sender: Sender<(String, Option<u8>)>) -> Player {
    let defaults : Stats = stats::from(&mut scanner::from(CharStream::from_file(File::open(DEFAULT).unwrap())));
    let mut identifiers : Stats = Stats::new();
    stats::set(&mut identifiers, "id", Value::Int(id as i64));
    stats::set(&mut identifiers, "name", Value::String(format!("Guest{}", id)));
    let mut posn : Stats = Stats::new();
    stats::set(&mut posn, "x", Value::Int(x as i64));
    stats::set(&mut posn, "y", Value::Int(y as i64));
    let mut data : Stats = Stats::new();
    stats::set(&mut data, "inventory", Value::Box(Stats::new()));
    stats::set(&mut data, "identity", Value::Box(identifiers));
    stats::set(&mut data, "base_stats", Value::Box(defaults.clone()));
    stats::set(&mut data, "stats", Value::Box(defaults.clone()));
    stats::set(&mut data, "posn", Value::Box(posn));
    stats::set(&mut data, "xp", Value::Int(0));
    return Player { data : data, sender : sender , opponent : None, equip : None, turn : false, interact: false, cumulative_speed : 0};
}

pub fn login(player : &mut Player, save : File) {
    let mut player_data : Stats = stats::from(&mut scanner::from(CharStream::from_file(save)));
    let a = stats::get(player.data(), "identity").unwrap().as_box();
    let id = stats::get(&a, "id").unwrap();
    let mut player_identifiers = stats::get(&player_data, "identity").unwrap().as_box();
    stats::set(&mut player_identifiers, "id", id.clone());
    stats::set(&mut player_data, "identity", Value::Box(player_identifiers));
    player.data = player_data;
}

pub fn respawn(player : &mut Player, world : &World) {
    let mut rng = rand::thread_rng();
    let mut x = rng.gen_range(0, world::MAP_SIZE);
    let mut y = rng.gen_range(0, world::MAP_SIZE);

    while stats::has_prop(world::get_block(world, x, y), "solid") {
        x = rng.gen_range(0, world::MAP_SIZE);
        y = rng.gen_range(0, world::MAP_SIZE);
    }
    set_posn(player, x, y);
    reset_to_base_with_buffs(player);
}

pub fn send(player : &Player, string : String) {
    player.sender.send((string, None)).unwrap();
}

pub fn change_stat(player : &mut Player, stat : &str, delta: i64) {
    let stat_val = get_stat(player, stat);
    let mut stats = stats::get(player.data(), "stats").unwrap().as_box();
    let base_stats = stats::get(player.data(), "base_stats").unwrap().as_box();
    let mut buff = 1.0f64;
    if player.equip.is_some() {
        let buffs = stats::get(player.equip().unwrap(), "buffs").unwrap().as_box();
        if stats::has_var(&buffs, stat) {
            buff = stats::get(&buffs, stat).unwrap().as_flt();
        }
    }
    let stat_max = ((stats::get(&base_stats, stat).unwrap().as_int() as f64) * buff) as i64;
    stats::set(&mut stats, stat, Value::Int(std::cmp::min(stat_max, std::cmp::max(0, stat_val + delta))));
    stats::set(&mut player.data, "stats", Value::Box(stats));
}

pub fn reset_to_base_with_buffs(player: &mut Player) {
    let mut stats = stats::get(player.data(), "stats").unwrap().as_box();
    let base_stats = stats::get(player.data(), "base_stats").unwrap().as_box();
    let stat_names = stats::get_var_names(&stats);
    let buffs;
    if player.equip.is_some() {
        buffs = stats::get(player.equip().unwrap(), "buffs").unwrap().as_box();
    } else {
        buffs = Stats::new();
    }
    for stat in stat_names {
        let mut base_stat = stats::get(&base_stats, stat.as_str()).unwrap().as_int();
        if stats::has_var(&buffs, stat.as_str()) {
            let buff = stats::get(&buffs, stat.as_str()).unwrap().as_flt();
            base_stat = ((base_stat as f64) * buff) as i64;
        }
        stats::set(&mut stats, stat.as_str(), Value::Int(base_stat));
    }
    stats::set(&mut player.data, "stats", Value::Box(stats));
}

pub fn change_xp(player : &mut Player, delta : i64) {
    let xp = stats::get(player.data(), "xp").unwrap().as_int();
    stats::set(&mut player.data, "xp", Value::Int(std::cmp::max(0, xp + delta)));
}

pub fn get_stat(player : &Player, stat : &str) -> i64 {
    let stats = stats::get(player.data(), "stats").unwrap().as_box();
    return stats::get(&stats, stat).unwrap().as_int();
}

pub fn xp(player : &Player) -> i64 {
    return stats::get(player.data(), "xp").unwrap().as_int();
}

pub fn x(player : &Player) -> u16 {
    return stats::get(&stats::get(player.data(), "posn").unwrap().as_box(), "x").unwrap().as_int() as u16;
}

pub fn y(player : &Player) -> u16 {
    return stats::get(&stats::get(player.data(), "posn").unwrap().as_box(), "y").unwrap().as_int() as u16;
}

pub fn set_posn(player: &mut Player, x : u16, y : u16) {
    let mut posn = stats::get(player.data(), "posn").unwrap().as_box();
    stats::set(&mut posn, "x", Value::Int(x as i64));
    stats::set(&mut posn, "y", Value::Int(y as i64));
    stats::set(&mut player.data, "posn", Value::Box(posn));
}

pub fn equip(player: &mut Player, item_name : String, world : &mut World) -> Result<(), String>{
    if !stats::has_var(&world.items, item_name.as_str()) {
        return Err("that item doesn't exist!".to_string());
    }
    let inventory = stats::get(player.data(), "inventory").unwrap().as_box();
    if !stats::has_var(&inventory, item_name.as_str()) {
        return Err("that item is not in your inventory!".to_string());
    }
    let item = stats::get(&world.items, item_name.as_str()).unwrap().as_box();
    player.equip = Some(item);
    return Ok(())
}

pub fn is_dead(player : &Player) -> bool {
    return get_stat(player, "health") == 0;
}

pub fn display(player: &Player) -> String {
    let id = format!("{:x}", stats::get(player.data(), "id").unwrap().as_int() as u8);
    if id.len() == 2 {
        return id;
    } else if id.len() == 1 {
        return format!("0{}", id);
    } else {
        unreachable!();
    }
}

pub fn add_items_to_inventory(player : &mut Player, stats : Stats) {
    let new_inventory = stats::add(stats::get(player.data(), "inventory").unwrap().as_box(), stats);
    stats::set(&mut player.data, "inventory", Value::Box(new_inventory));
}

pub fn remove_item_from_inventory(player : &mut Player, item : &str) {
    let mut inv = stats::get(player.data(), "inventory").unwrap().as_box();
    if stats::has_var(&inv, item) {
        let m = stats::get(&mut inv, item).unwrap().as_int();
        stats::set(&mut inv, item, Value::Int(m - 1));
        if stats::get(&mut inv, item).unwrap().as_int() == 0 {
            stats::rm(&mut inv, item);
        }
    }
    stats::set(&mut player.data, "inventory", Value::Box(inv));
}

pub fn get_inventory(player : &Player) -> Stats {
    return stats::get(player.data(), "inventory").unwrap().as_box();
}

pub fn turn(player : &Player) -> bool {
    return player.turn;
}

pub fn set_turn(player : &mut Player, val : bool) {
    player.turn = val;
}

pub fn upgrade_stat(player : &mut Player, stat : &str) -> Result<(), String> {
    let mut base_stats = stats::get(player.data(), "base_stats").unwrap().as_box();
    let val = stats::get(&base_stats, stat).unwrap().as_int();
    let xp_cost = val * 100;
    if xp_cost > xp(player) {
        return Err(format!("you need {} xp to upgrade this stat, but you only have {} xp.", xp_cost, xp(player)));
    }
    change_xp(player, -xp_cost);
    stats::set(&mut base_stats, stat, Value::Int(val + 1));
    stats::set(&mut player.data, "base_stats", Value::Box(base_stats));
    return Ok(());
}
