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
use std::error::Error;
use crate::playerout::PlayerOut;

const DEFAULT : &str = "config/player_defaults.txt";

pub struct Player {
    data : Stats,
    opponent : Option<u8>,
    equip : Option<Stats>,
    wears : Vec<(String, Stats)>,
    sender : Sender<(PlayerOut, Option<u8>)>,
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
            return Some(self.equip.as_ref()?);
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

fn buff(player : &mut Player, buff : Stats) -> Result<(), Box<dyn Error>> {
    let mut base = stats::get(player.data(), "buffed_stats")?.as_box()?;
    let vars = stats::get_var_names(&base);
    for var in vars {
        let curr = stats::get_or_else(&base, var.as_str(), &Value::Int(0)).as_int()?;
        if stats::has_var(&buff, var.as_str()) {
            let b = stats::get(&buff, var.as_str())?.as_flt()?;
            stats::set(&mut base, var.as_str(), stats::Value::Int(((curr as f64) * b) as i64));
        }
    }
    stats::set(&mut player.data, "buffed_stats", Value::Box(base));
    return Ok(())
}

pub fn wear(player : &mut Player, name : String, b : Stats) -> Result<(), Box<dyn Error>> {
    let inv = get_inventory(player)?;
    if !stats::has_var(&inv, &name) {
        return Err("You don't have that item in your inventory!".into());
    }
    buff(player, b.clone())?;
    remove_item_from_inventory(player, &name)?;
    player.wears.push((name, b));
    return Ok(());
}

pub fn unwear_all(player : &mut Player) -> Result<(), Box<dyn Error>> {
    unwear(player)?;
    let mut names = vec![];
    for (name, _) in &player.wears {
        names.push(name.clone());
    }
    for name in names {
        add_item_to_inventory(player, &name)?;
    }
    player.wears = vec![];
    return Ok(())
}

fn unwear(player : &mut Player) -> Result<(), Box<dyn Error>> {
    let base_stats = stats::get(player.data(), "base_stats")?.as_box()?;
    stats::set(&mut player.data, "buffed_stats", Value::Box(base_stats));
    reset_to_base(player)?;
    return Ok(());
}

pub fn from(x : u16, y : u16, id : u8, sender: Sender<(PlayerOut, Option<u8>)>) -> Result<Player, Box<dyn Error>> {
    let defaults : Stats = stats::from(&mut scanner::from(CharStream::from_file(File::open(DEFAULT)?)))?;
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
    stats::set(&mut data, "buffed_stats", Value::Box(defaults.clone()));
    stats::set(&mut data, "stats", Value::Box(defaults.clone()));
    stats::set(&mut data, "posn", Value::Box(posn));
    stats::set(&mut data, "xp", Value::Int(0));
    return Ok(Player { 
                data : data, 
                sender : sender, 
                opponent : None, 
                equip : None, 
                wears : vec![],
                turn : false, 
                interact: false, 
                cumulative_speed : 0
            });
}

/*
pub fn login(player : &mut Player, save : File) -> Result<(), Box<dyn Error>> {
    let mut player_data : Stats = stats::from(&mut scanner::from(CharStream::from_file(save)));
    let a = stats::get(player.data(), "identity")?.as_box()?;
    let id = stats::get(&a, "id")?;
    let mut player_identifiers = stats::get(&player_data, "identity")?.as_box()?;
    stats::set(&mut player_identifiers, "id", id.clone());
    stats::set(&mut player_data, "identity", Value::Box(player_identifiers));
    player.data = player_data;
}
*/

pub fn respawn(player : &mut Player, world : &World) -> Result<(), Box<dyn Error>> {
    let mut rng = rand::thread_rng();
    let mut x = rng.gen_range(0, world::MAP_SIZE);
    let mut y = rng.gen_range(0, world::MAP_SIZE);

    while stats::has_prop(world::get_block(world, x, y)?, "solid") {
        x = rng.gen_range(0, world::MAP_SIZE);
        y = rng.gen_range(0, world::MAP_SIZE);
    }
    set_posn(player, x, y)?;
    reset_to_base(player)?;
    let health = stats::get(&stats::get(player.data(), "buffed_stats")?.as_box()?, "health")?.as_int()?;
    change_stat(player, "health", health)?;
    return Ok(())
}

pub fn send(player : &Player, out : PlayerOut) -> Result<(), Box<dyn Error>> {
    let res = player.sender.send((out, None));
    match res {
        Ok(_) => Ok(()),
        Err(_) => Err("send failure".into())
    }
}

pub fn send_str<S : Into<String>>(player : &Player, text : S) -> Result<(), Box<dyn Error>> {
    let mut out = PlayerOut::new();
    out.append(text);
    return send(player, out);
}

pub fn change_stat(player : &mut Player, stat : &str, delta: i64) -> Result<(), Box<dyn Error>> {
    let stat_val = get_stat(player, stat)?;
    let mut stats = stats::get(player.data(), "stats")?.as_box()?;
    let base_stats = stats::get(player.data(), "buffed_stats")?.as_box()?;
    let stat_max = stats::get(&base_stats, stat)?.as_int()?;
    stats::set(&mut stats, stat, Value::Int(std::cmp::min(stat_max, std::cmp::max(0, stat_val + delta))));
    stats::set(&mut player.data, "stats", Value::Box(stats));
    return Ok(());
}

pub fn reset_to_base(player: &mut Player) -> Result<(), Box<dyn Error>> {
    let health = get_stat(player, "health")?;
    let mut base_stats = stats::get(player.data(), "buffed_stats")?.as_box()?;
    let max_health = stats::get(&base_stats, "health")?.as_int()?;
    stats::set(&mut base_stats, "health", Value::Int(std::cmp::min(health, max_health)));
    stats::set(&mut player.data, "stats", Value::Box(base_stats));
    return Ok(());
}

pub fn change_xp(player : &mut Player, delta : i64) -> Result<(), Box<dyn Error>> {
    let xp = stats::get(player.data(), "xp")?.as_int()?;
    stats::set(&mut player.data, "xp", Value::Int(std::cmp::max(0, xp + delta)));
    return Ok(());
}

pub fn get_stat(player : &Player, stat : &str) -> Result<i64, Box<dyn Error>> {
    let stats = stats::get(player.data(), "stats")?.as_box()?;
    return stats::get(&stats, stat)?.as_int();
}

pub fn xp(player : &Player) -> Result<i64, Box<dyn Error>> {
    return stats::get(player.data(), "xp")?.as_int();
}

pub fn x(player : &Player) -> Result<u16, Box<dyn Error>> {
    return Ok(stats::get(&stats::get(player.data(), "posn")?.as_box()?, "x")?.as_int()? as u16);
}

pub fn y(player : &Player) -> Result<u16, Box<dyn Error>> {
    return Ok(stats::get(&stats::get(player.data(), "posn")?.as_box()?, "y")?.as_int()? as u16);
}

pub fn set_posn(player: &mut Player, x : u16, y : u16) -> Result<(), Box<dyn Error>> {
    let mut posn = stats::get(player.data(), "posn")?.as_box()?;
    stats::set(&mut posn, "x", Value::Int(x as i64));
    stats::set(&mut posn, "y", Value::Int(y as i64));
    stats::set(&mut player.data, "posn", Value::Box(posn));
    return Ok(())
}

pub fn is_dead(player : &Player) -> Result<bool, Box<dyn Error>> {
    return Ok(get_stat(player, "health")? == 0);
}

pub fn add_items_to_inventory(player : &mut Player, stats : Stats) -> Result<(), Box<dyn Error>> {
    let new_inventory = stats::add(stats::get(player.data(), "inventory")?.as_box()?, stats)?;
    stats::set(&mut player.data, "inventory", Value::Box(new_inventory));
    return Ok(());
}

pub fn remove_item_from_inventory(player : &mut Player, item : &str) -> Result<(), Box<dyn Error>> {
    let mut inv = stats::get(player.data(), "inventory")?.as_box()?;
    if stats::has_var(&inv, item) {
        let m = stats::get(&mut inv, item)?.as_int()?;
        stats::set(&mut inv, item, Value::Int(m - 1));
        if stats::get(&mut inv, item)?.as_int()? == 0 {
            stats::rm(&mut inv, item);
        }
    }
    stats::set(&mut player.data, "inventory", Value::Box(inv));
    return Ok(());
}

pub fn add_item_to_inventory(player : &mut Player, item : &str) -> Result<(), Box<dyn Error>> {
    let mut stats = Stats::new();
    stats::set(&mut stats, item, stats::Value::Int(1));
    add_items_to_inventory(player, stats)?;
    return Ok(());
}

pub fn get_inventory(player : &Player) -> Result<Stats, Box<dyn Error>> {
    return Ok(stats::get(player.data(), "inventory")?.as_box()?);
}

pub fn turn(player : &Player) -> bool {
    return player.turn;
}

pub fn set_turn(player : &mut Player, val : bool) {
    player.turn = val;
}

pub fn upgrade_stat(player : &mut Player, stat : &str) -> Result<(), Box<dyn Error>> {
    let mut base_stats = stats::get(player.data(), "base_stats")?.as_box()?;
    let val = stats::get(&base_stats, stat)?.as_int()?;
    let xp_cost = val * 100;
    if xp_cost > xp(player)? {
        return Err(format!("you need {} xp to upgrade this stat, but you only have {} xp.", xp_cost, xp(player)?).into());
    }
    change_xp(player, -xp_cost)?;
    unwear(player)?;
    stats::set(&mut base_stats, stat, Value::Int(val + 1));
    stats::set(&mut player.data, "base_stats", Value::Box(base_stats.clone()));
    stats::set(&mut player.data, "buffed_stats", Value::Box(base_stats.clone()));
    for i in 0..player.wears.len() {
        let name = player.wears[i].0.clone();
        let stat = player.wears[i].1.clone();
        wear(player, name, stat)?;
    }
    reset_to_base(player)?;
    return Ok(());
}
