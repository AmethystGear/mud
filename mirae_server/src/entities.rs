extern crate rand;
extern crate rstring_builder;

use crate::stats::Value;
use crate::player::Player;
use crate::world;
use crate::world::World;
use std::collections::HashMap;
use crate::stats;
use crate::stats::Stats;
use crate::action;
use crate::action::{Action, ActionMap, ActionFunc};
use crate::scanner::Param;
use rand::Rng;
use crate::{playerout::PlayerOut, player};
use std::error::Error;
use rstring_builder::StringBuilder;

type Res = std::result::Result<PlayerOut, Box<dyn Error>>;
type Create = Result<Box<dyn Spawnable>, Box<dyn Error>>;

pub struct SpawnedEntities {
    spawned_entities : HashMap<u32, Box<dyn Spawnable>>,
    create_entity_map : HashMap<String, fn(stats::Stats, u16, u16, String, &World) -> Create>,
    entity_actions_map: HashMap<String, ActionMap>
}

impl SpawnedEntities {
    pub fn new() -> SpawnedEntities {
        let mut create = HashMap::new();
        create.insert("mob".to_string(), create_mob as fn(stats::Stats, u16, u16, String, &World) -> Create);
        create.insert("trading_mob".to_string(), create_trading_mob as fn(stats::Stats, u16, u16, String, &World) -> Create);
        create.insert("chest".to_string(), create_chest as fn(stats::Stats, u16, u16, String, &World) -> Create);

        let mut chest = ActionMap::new();
        action::add_action(&mut chest, "interact".to_string(), Action::new(ActionFunc::F(chest_interact)));

        let mut mob = ActionMap::new();
        action::add_action(&mut mob, "dmg".to_string(), Action::new(ActionFunc::G(dmg)));
        action::add_action(&mut mob, "interact".to_string(), Action::new(ActionFunc::F(interact_mob)));
        action::add_action(&mut mob, "attack".to_string(), Action::new(ActionFunc::F(attack)));

        let mut trading_mob = ActionMap::new();
        action::add_action(&mut trading_mob, "dmg".to_string(), Action::new(ActionFunc::G(dmg)));
        action::add_action(&mut trading_mob, "interact".to_string(), Action::new(ActionFunc::F(interact_mob)));
        action::add_action(&mut trading_mob, "attack".to_string(), Action::new(ActionFunc::F(attack)));
        action::add_action(&mut trading_mob, "trade".to_string(), Action::new(ActionFunc::G(trade)));

        let mut entity_actions_map = HashMap::new();
        entity_actions_map.insert("chest".to_string(), chest);
        entity_actions_map.insert("mob".to_string(), mob);
        entity_actions_map.insert("trading_mob".to_string(), trading_mob);
        SpawnedEntities {
            spawned_entities : HashMap::new(),
            create_entity_map : create,
            entity_actions_map : entity_actions_map
        }
    }
}

pub trait Spawnable {
    fn x(&self) -> u16;
    fn y(&self) -> u16;
    fn mut_data(&mut self) -> &mut Stats;
    fn data(&self) -> Stats;
    fn name(&self) -> String;
    fn entity_type(&self) -> String;
}

pub fn hash(x : u16, y : u16) -> u32 {
    return ((y as u32) << 16) | (x as u32);
}

pub fn spawn(stats: Stats, x: u16, y: u16, name : String, se: &mut SpawnedEntities, world : &mut World) -> Result<(), Box<dyn Error>> {
    let entity_type = stats::get(&stats, "entity_type")?.as_string()?;
    let create_entity = 
                    se.create_entity_map.get(&entity_type)
                    .ok_or(format!("could not find create entity function for {}", name))?;

    let entity = create_entity(stats, x, y, name, world)?;
    se.spawned_entities.insert(hash(x, y), entity);
    return Ok(())
}

pub fn has_entity(spawned_entities : &SpawnedEntities, x : u16, y : u16) -> bool {
    return spawned_entities.spawned_entities.contains_key(&hash(x, y));
}

pub fn get_entity_action(spawned_entities : &SpawnedEntities, keyword : String, x : u16, y : u16) -> Option<Action> {
    let entity = get_entity(spawned_entities, x, y);
    if entity.is_none() {
        return None;
    }
    let entity_type = entity?.entity_type();
    let entity_action_map = spawned_entities.entity_actions_map.get(&entity_type)?;
    let u = action::get_action_and_params(entity_action_map, keyword);
    match u {
        Ok(u) => {
            let (_, _, action) = u;
            return Some(action)
        },
        Err(_) => return None
    }
}

pub fn get_entity_mut(se : &mut SpawnedEntities, x : u16, y : u16) -> Option<&mut dyn Spawnable> {
    let e = se.spawned_entities.get_mut(&hash(x, y));
    return Some(e?.as_mut());
}

pub fn get_entity(se : &SpawnedEntities, x : u16, y : u16) -> Option<&dyn Spawnable> {
    let e = se.spawned_entities.get(&hash(x, y));
    return Some(e?.as_ref());
}

pub fn remove_entity(w : &mut World, se : &mut SpawnedEntities, x : u16, y : u16) {
    se.spawned_entities.remove(&hash(x, y));
    world::remove_entity(w, x, y);
}

pub struct Mob {
    x : u16,
    y : u16,
    entity_type : String,
    base_stats : Stats,
    name : String,
    stats : Stats,
    weapons : Stats,
}

impl Spawnable for Mob {
    fn x(&self) -> u16 {
        return self.x;
    }
    fn y(&self) -> u16 {
        return self.y;
    }
    fn mut_data(&mut self) -> &mut Stats {
        return &mut self.stats;
    }
    fn data(&self) -> Stats {
        let mut stats = self.base_stats.clone();
        stats::set(&mut stats, "weapons", stats::Value::Box(self.weapons.clone()));
        return stats;
    }
    fn name(&self) -> String {
        return self.name.clone();
    }
    fn entity_type(&self) -> String {
        return self.entity_type.clone();
    }
}

impl Mob {
    pub fn new(stats: Stats, x : u16, y : u16, name : String) -> Result<Self, Box<dyn Error>> {
        let s =  stats::get(&stats, "stats")?.as_box()?;
        let mut m = Mob {
            x : x,
            y : y,
            base_stats : stats,
            name : name,
            stats : s,
            entity_type : "mob".to_string(),
            weapons : Stats::new()
        };
        m.weapons = get_items(&m, "item")?;
        return Ok(m);
    }
}

pub fn create_mob(stats: Stats, x: u16, y: u16, name : String, _world: &World) -> Result<Box<dyn Spawnable>, Box<dyn Error>> {
    let mob = Mob::new(stats, x, y, name)?;
    return Ok(Box::new(mob));
}

pub struct TradingMob {
    x : u16,
    y : u16,
    base_stats : Stats,
    stats : Stats,
    items : Stats,
    entity_type : String,
    name : String,
    weapons : Stats
}

impl Spawnable for TradingMob {
    fn x(&self) -> u16 {
        return self.x;
    }
    fn y(&self) -> u16 {
        return self.y;
    }
    fn mut_data(&mut self) -> &mut Stats {
        return &mut self.stats;
    }
    fn data(&self) -> Stats {
        let mut stats = self.stats.clone();
        stats::set(&mut stats, "items", stats::Value::Box(self.items.clone()));
        stats::set(&mut stats, "weapons", stats::Value::Box(self.weapons.clone()));
        stats::set(&mut stats, "base_stats", stats::Value::Box(self.base_stats.clone()));
        return stats;
    }
    fn name(&self) -> String {
        return self.name.clone();
    }
    fn entity_type(&self) -> String {
        return self.entity_type.clone();
    }
}

impl TradingMob {
    pub fn new (stats: Stats, x : u16, y : u16, name : String, world : &World) -> Result<Self, Box<dyn Error>> {
        let min = stats::get(&stats, "trade_min")?.as_int()? as usize;
        let max = stats::get(&stats, "trade_max")?.as_int()? as usize;
        let mut rng = rand::thread_rng();
        let num_items = rng.gen_range(min, max + 1);
        let items = get_random_items(num_items, world)?;
        let s = stats::get(&stats, "stats")?.as_box()?;
        let mut tm = TradingMob {
            x : x,
            y : y,
            base_stats: stats,
            stats : s,
            name : name,
            items : items,
            entity_type : "trading_mob".to_string(),
            weapons : Stats::new()
        };
        tm.weapons = get_items(&tm, "item")?;
        return Ok(tm);
    }
}

pub fn create_trading_mob(stats: Stats, x: u16, y: u16, name : String, world: &World) -> Create {
    return Ok(Box::new(TradingMob::new(stats, x, y, name, world)?));
}

pub struct LootChest {
    x : u16,
    y : u16,
    name : String,
    items : Stats,
}

impl Spawnable for LootChest {
    fn x(&self) -> u16 {
        return self.x;
    }
    fn y(&self) -> u16 {
        return self.y;
    }
    fn mut_data(&mut self) -> &mut Stats {
        return &mut self.items;
    }
    fn data(&self) -> Stats {
        return self.items.clone();
    }
    fn name(&self) -> String {
        return self.name.clone();
    }
    fn entity_type(&self) -> String {
        return "chest".to_string();
    }
}

impl LootChest {
    pub fn new (stats: Stats, x : u16, y : u16, name : String, world : &World) -> Result<Self, Box<dyn Error>> {
        let min = stats::get_or_else(&stats, "items_min", &stats::Value::Int(0)).as_int()? as usize;
        let max = stats::get_or_else(&stats, "items_max", &stats::Value::Int(0)).as_int()? as usize;
        let mut rng = rand::thread_rng();
        let num_items = rng.gen_range(min, max + 1);
        let items = get_random_items(num_items, world)?;
        return Ok(LootChest {
            x,
            y,
            name,
            items
        });
    }
}

pub fn get_random_items(num_items : usize, world : &World) -> Result<Stats, Box<dyn Error>> {
    let mut items = Stats::new();
    for _ in 0..num_items {
        let item = world::get_random_item(world);
        if !stats::has_var(&items, &item) {
            stats::set(&mut items, &item, stats::Value::Int(1));
        } else {
            let clone = items.clone();
            stats::set(&mut items, &item, stats::Value::Int(stats::get(&clone, &item)?.as_int()? + 1));
        }
    }
    return Ok(items);
}

pub fn create_chest(stats : Stats, x : u16, y : u16, name : String, world : &World) -> Create {
    return Ok(Box::new(LootChest::new(stats, x, y, name, world)?));
}

pub fn chest_interact(entities : &mut SpawnedEntities, player_id : u8, players : &mut Vec<Option<Player>>, world : &mut World) -> Res {
    let player = players[player_id as usize].as_mut().ok_or("player id invalid")?;
    let x = player::x(&player)?;
    let y = player::y(&player)?;
    let chest = get_entity(entities, x, y).ok_or("can't get entity")?;
    let mut out = PlayerOut::new();
    out.append("you encountered a ");
    out.append(chest.name());
    out.append("\n");
    out.append("you recieved:\n");
    out.append(stats::string(&chest.data())?);
    player::add_items_to_inventory(player, chest.data().clone())?;
    remove_entity(world, entities, x, y);
    return Ok(out);
}

pub fn dmg(entities : &mut SpawnedEntities, params : &Vec<Param>, player_id : u8, players : &mut Vec<Option<Player>>, world : &mut World) -> Res {
    let player = players[player_id as usize].as_mut().ok_or("player_id invalid")?;
    let x = player::x(&player)?;
    let y = player::y(&player)?;
    let entity = get_entity_mut(entities, x, y).ok_or("no spawned entity at location!")?;
    let physical_dmg = params[0].as_int()?;
    let magic_dmg = params[1].as_int()?;
    let entity_name = entity.name();
    let data = entity.mut_data();
    let phys_def = stats::get_or_else(data, "physical_def", &stats::Value::Float(0.0f64)).as_flt()?;
    let magic_def = stats::get_or_else(data, "magic_def", &stats::Value::Float(0.0f64)).as_flt()?;
    let true_physical_dmg = ((physical_dmg as f64) * (1.0f64 - phys_def)) as i64;
    let true_magic_dmg = ((magic_dmg as f64) * (1.0f64 - magic_def)) as i64;
    let dmg = true_magic_dmg + true_physical_dmg;
    let health = stats::get(data, "health")?.as_int()?;
    let mut out = PlayerOut::new();
    out.append(format!("{} took {} damage!\n", entity_name, dmg));
    if health <= dmg {
        stats::set(data, "health", stats::Value::Int(0));
        out.append(format!("{}: {}\n", entity.name(), get_random_quote(entity, "player_victory")?));
        out.append(format!("You MURDERED {}! CONGRATULATIONS!\n", entity.name()));
        if stats::has_var(&entity.data(), "drops") {
            let mob_drops = get_items(entity, "drop")?;
            out.append("you got:\n");
            out.append(stats::string(&mob_drops)?);
            out.append("and:\n");
            let def = Value::Int(0);
            let xp = stats::get(&stats::get(&entity.data(), "stats")?.as_box()?, "xp").unwrap_or(&def).as_int()?;
            out.append(format!("{} xp.\n", xp));
            let player = players[player_id as usize].as_mut().ok_or("")?;
            player::add_items_to_inventory(player, mob_drops)?;
            player::change_xp(player, xp)?;
        }
        remove_entity(world, entities, x, y);
    } else {
        stats::set(data, "health", stats::Value::Int(health - dmg));
    }
    return Ok(out);
}

pub fn trade(entities : &mut SpawnedEntities, params : &Vec<Param>, player_id : u8, players : &mut Vec<Option<Player>>, world : &mut World) -> Res {
    let player = players[player_id as usize].as_mut().ok_or("can't get player!")?;
    let x = player::x(&player)?;
    let y = player::y(&player)?;
    let entity = get_entity(entities, x, y).ok_or("")?;

    let mut out = PlayerOut::new();
    let items = stats::get(&entity.data(), "items")?.as_box()?;
    let item_names = stats::get_var_names(&items);
    if params.is_empty() {
        out.append("the availible trades are:\n");
        for i in 0..item_names.len() {
            let item = &item_names[i];
            let item_box = stats::get(&world.items(), item)?.as_box()?;
            let xp = stats::get_or_else(&item_box, "xp", &stats::Value::Int(0)).as_int()?;
            out.append(format!("{}. {} --> {} xp\n", i, item, xp));
        }
    } else if params.len() == 2 {
        if params[0].as_int().is_err() || params[1].as_int().is_err() {
            return Err("expected 2 integers as parameters".into());
        }
        let trade_num = params[0].as_int()?;
        let num_to_trade = params[1].as_int()?;
        let player = players[player_id as usize].as_mut().ok_or("player id is invalid")?;
        let inventory = stats::get(player.data(), "inventory")?.as_box()?;
        if trade_num < 0 || trade_num > item_names.len() as i64 {
            return Err(format!("there is no trade numbered {}", trade_num).into());
        }
        let item = &item_names[trade_num as usize];
        let num_in_inventory = stats::get_or_else(&inventory, item, &stats::Value::Int(0)).as_int()?;
        if num_to_trade > num_in_inventory {
            return Err(format!("you only have {} of that item", num_in_inventory).into());
        }
        let item_box = stats::get(&world.items(), item.as_str())?.as_box()?;
        let xp = stats::get_or_else(&item_box, "xp", &stats::Value::Int(0)).as_int()?;
        player::change_xp(player, num_to_trade * xp)?;
        out.append(format!("You traded {} of {} for {} xp. This is the best trade deal in the history of trade deals, maybe ever.\n", num_to_trade, item, num_to_trade * xp));
    } else {
        return Err("expected 0 or 2 parameters".into());
    }
    return Ok(out);
}

pub fn interact_mob(entities : &mut SpawnedEntities, player_id : u8, players: &mut Vec<Option<Player>>, world : &mut World) -> Res {
    let mut out = PlayerOut::new();
    let player = players[player_id as usize].as_mut().ok_or("player id is invalid")?;
    let x = player::x(&player)?;
    let y = player::y(&player)?;
    let entity = get_entity_mut(entities, x, y).ok_or("entity doesn't exist at that position!")?;
    let entrance_quote = get_random_quote(entity, "entrance")?;
    out.append(format!("You have encountered {}\n", entity.name()));
    out.append(format!("{}: {}\n", entity.name(), entrance_quote));

    let img = stats::get_or_else(&entity.data(), "img", 
                        &stats::Value::LongString(stats::StrBuilder::new(StringBuilder::new()))).as_longstring()?;

    out.append(img.string());
    let stats = stats::get(player.data(), "stats")?.as_box()?;
    let player_speed = stats::get(&stats, "speed")?.as_int()?;
    let entity_speed = stats::get_or_else(entity.mut_data(), "speed", &stats::Value::Int(0)).as_int()?;
    if stats::has_prop(&entity.data(), "attack_first") && entity_speed > player_speed {
        out.append_player_out(attack(entities, player_id, players, world)?);
    }
    return Ok(out);
}

pub fn attack(entity: &mut SpawnedEntities, player_id : u8, players: &mut Vec<Option<Player>>, world : &mut World) -> Res {
    let player = players[player_id as usize].as_mut().ok_or("player id is invalid")?;
    let x = player::x(&player)?;
    let y = player::y(&player)?;
    let entity = get_entity_mut(entity, x, y).ok_or("entity doesn't exist")?;

    let mut out = PlayerOut::new();
    let mut cumulative_player_speed = 0;
    let entity_speed = stats::get_or_else(entity.mut_data(), "speed", &stats::Value::Int(0)).as_int()?;
    let base_dmg = stats::get_or_else(entity.mut_data(), "dmg", &stats::Value::Int(0)).as_int()?;
    let stats = stats::get(player.data(), "stats")?.as_box()?;
    let player_speed = stats::get(&stats, "speed")?.as_int()?;
    println!("{}", player_speed);
    while cumulative_player_speed < entity_speed {
        cumulative_player_speed += player_speed;

        println!("{}, {}",  cumulative_player_speed, entity_speed);
        let attack_quote = get_random_quote(entity, "attack")?;
        out.append(format!("{}: {}\n", entity.name(), attack_quote));
        player::change_stat(player, "health", -base_dmg)?;
        out.append(format!("{} did {} damage to you.\n", entity.name(), base_dmg));
        let weapons = stats::get_var_names(&stats::get(&entity.data(), "weapons")?.as_box()?);
        if !weapons.is_empty() {
            let mut rng = rand::thread_rng();
            let weapon_name = &weapons[rng.gen_range(0, weapons.len())];
            let weapon = stats::get_or_else(&world.items(), weapon_name.as_str(), &Value::Box(Stats::new())).as_box()?;
            if stats::has_var(&weapon, "abilities") {
                out.append(format!("{} equipped item {}\n", entity.name(), weapon_name));
                let abilities = stats::get(&weapon, "abilities")?.as_box()?;
                let ability_names = stats::get_var_names(&abilities);
                let ability_name = &ability_names[rng.gen_range(0, ability_names.len())];
                let ability = stats::get(&abilities, ability_name.as_str())?.as_box()?;
                let mut physical_dmg = 0;
                let mut magic_dmg = 0;
                if stats::has_var(&ability, "physical_dmg") {
                    physical_dmg = stats::get(&ability, "physical_dmg")?.as_int()?;
                }
                if stats::has_var(&ability, "magic_dmg") {
                    magic_dmg = stats::get(&ability, "magic_dmg")?.as_int()?;
                }
                player::change_stat(player, "health", -(physical_dmg + magic_dmg))?;
                out.append(format!("{} used {} and dealt {} damage.\n", entity.name(), ability_name, (physical_dmg + magic_dmg)));
            } else {
                out.append(format!("{} chose to use {}, which has no abilities. Lucky you.\n", entity.name(), weapon_name));
            }
        }
        if player::is_dead(&player)? {
            out.append("YOU DIED.\n");
            out.append("respawning...\n");
            player::respawn(player, world)?;
            break;
        }
    }
    return Ok(out);
}

pub fn get_random_quote(entity: &dyn Spawnable, quote_name : &str) -> Result<String, Box<dyn Error>> {
    let quotes = stats::get_or_else(&entity.data(), "quotes", &stats::Value::Box(Stats::new())).as_box()?;
    let mut rng = rand::thread_rng();
    if stats::has_var(&quotes, quote_name) {
        let player_win_quotes = stats::get(&quotes, quote_name)?.as_vec()?;
        return player_win_quotes[rng.gen_range(0, player_win_quotes.len())].as_string();
    } else {
        return Ok("".to_owned());
    }
}

pub fn get_items(entity : &dyn Spawnable, item : &str) -> Result<Stats, Box<dyn Error>> {
    if !stats::has_var(&entity.data(),  &format!("{}s", item)) {
        return Ok(Stats::new());
    }
    let drops = stats::get(&entity.data(), &format!("{}s", item))?.as_box()?;
    let drop_names = stats::get(&drops, &format!("{}s", item))?.as_vec()?;
    let probs = stats::get(&drops, &format!("{}_prob", item))?.as_vec()?;
    let default : Vec<Value> = vec![stats::Value::Int(1i64); drop_names.len()];
    let drop_per = stats::get_or_else(&drops, &format!("{}_per", item), &stats::Value::List(default)).as_vec()?;

    let mut rng = rand::thread_rng();
    let min = stats::get_or_else(&drops, &format!("{}_min", item), &stats::Value::Int(0)).as_int()? as usize;
    let max = stats::get_or_else(&drops, &format!("{}_max", item), &stats::Value::Int(0)).as_int()? as usize;
    let num_runs = rng.gen_range(min, max + 1);
    println!("num_runs: {}", num_runs);

    let mut mob_drops = Stats::new();
    let mut thresholds = vec![];
    let mut sum = 0.0f64;
    for prob in probs {
        sum += prob.as_flt()?;
        thresholds.push(sum);
    }

    for _ in 0..num_runs {
        let p : f64 = rng.gen();
        for i in 0..thresholds.len() {
            if p < thresholds[i] {
                stats::set(&mut mob_drops, drop_names[i].as_string()?.as_str(), drop_per[i].clone());
                break;
            }
        }
    }
    println!("mob drops:\n {}", stats::string(&mob_drops)?);
    return Ok(mob_drops);
}
