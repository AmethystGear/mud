#![allow(dead_code)]
use actions::{dispatch, get_mut, get_two_mut, ActionData};
use anyhow::{anyhow, Error, Result};
use combat::{BattleHandle, BattleMap, EntityType, ID};
use display::{Bounds, Image};
use entity::Entity;
use gamedata::gamedata::{GameData, GameMode};
use image::{ImageBuffer, Rgb};
use io::{BufRead, Read, Write};
use player::Player;
use playerout::PlayerOut;
use rand::{prelude::StdRng, thread_rng, Rng, SeedableRng};
use serde_jacl::{
    de::from_str,
    structs::{Literal, Number},
};
use std::{
    collections::{HashMap, VecDeque},
    env, fs, io,
    iter::FromIterator,
    net::TcpStream,
    sync::{Arc, RwLock},
    thread::{self, spawn},
    time::{self, Instant},
};
use vector3::Vector3;
use websocket::{
    sync::{Client, Server},
    OwnedMessage,
};
use world::World;

use crossbeam::channel::{unbounded, Sender};
use fs::{File, OpenOptions};
use rgb::RGB;
use time::Duration;

mod actions;
mod combat;
mod display;
mod entity;
mod gamedata;
mod inventory;
mod mob;
mod noise;
mod player;
mod playerout;
mod rgb;
mod stat;
mod vector3;
mod world;

const PLAYER_SAVE_FOLDER : &str = "save/player_save";
const WORLD_SAVE_FOLDER : &str = "save/world_save";
const DEBUG_BLOCK_SIZE: u32 = 10;

fn save_img(image: Image, save_location: &str) -> Result<()> {
    let width = (image.width as u32) * DEBUG_BLOCK_SIZE;
    let height = (image.height as u32) * DEBUG_BLOCK_SIZE;
    let mut image_out = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(width, height);

    for y in 0..(image.height as u32) {
        for x in 0..(image.width as u32) {
            let index = ((x as usize) + (y as usize) * (image.width as usize)) * 3;
            let r = image.rgb[index];
            let g = image.rgb[index + 1];
            let b = image.rgb[index + 2];
            for j in 0..DEBUG_BLOCK_SIZE {
                for i in 0..DEBUG_BLOCK_SIZE {
                    let x = x * DEBUG_BLOCK_SIZE + i;
                    let y = y * DEBUG_BLOCK_SIZE + j;
                    image_out.get_pixel_mut(x, y).0 = [r, g, b];
                }
            }
        }
    }
    image_out.save(save_location)?;
    Ok(())
}

fn map(
    mut params: VecDeque<Literal>,
    world: &World,
    players: &Vec<Option<Player>>,
    g: &GameData,
) -> Result<()> {
    // parse in our parameters
    let size;
    let layer;
    let save_location;
    let help = "\"map\" <map_size> <layer> <location to save image>";
    match (params.pop_front(), params.pop_front(), params.pop_front()) {
        (
            Some(Literal::Number(Number::Int(i0))),
            Some(Literal::Number(Number::Int(i1))),
            Some(Literal::String(s)),
        ) => {
            size = i0;
            layer = i1;
            save_location = s;
        }
        _ => return Err(anyhow!(help)),
    }

    let resolution = world.blocks().dim.x() / (size as isize);
    let bounds = Bounds::get_bounds(
        &world,
        Vector3::new(0, 0, layer as isize),
        world.blocks().dim.x() as usize,
        world.blocks().dim.y() as usize,
    );
    let image = Image::new(&world, &players, &g, &bounds, resolution as usize)?;
    save_img(image, &save_location)?;
    Ok(())
}

fn look(
    mut params: VecDeque<Literal>,
    world: &World,
    players: &Vec<Option<Player>>,
    g: &GameData,
) -> Result<()> {
    let x;
    let y;
    let z;
    let width;
    let height;
    let save_location;
    let help = "\"look\" <x> <y> <z> <height> <width> <location to save image>";
    match (
        params.pop_front(),
        params.pop_front(),
        params.pop_front(),
        params.pop_front(),
        params.pop_front(),
        params.pop_front(),
    ) {
        (
            Some(Literal::Number(Number::Int(i0))),
            Some(Literal::Number(Number::Int(i1))),
            Some(Literal::Number(Number::Int(i2))),
            Some(Literal::Number(Number::Int(i3))),
            Some(Literal::Number(Number::Int(i4))),
            Some(Literal::String(s)),
        ) => {
            x = i0;
            y = i1;
            z = i2;
            width = i3;
            height = i4;
            save_location = s;
        }
        _ => return Err(anyhow!(help)),
    }

    let posn = Vector3::new(x as isize, y as isize, z as isize);
    let bounds = Bounds::get_bounds(&world, posn, width as usize, height as usize);
    let image = Image::new(&world, &players, &g, &bounds, 1)?;
    save_img(image, &save_location)?;
    Ok(())
}

fn save_world(
    mut params: VecDeque<Literal>,
    world: &World,
    _players: &Vec<Option<Player>>,
    g: &GameData,
) -> Result<()> {
    let help = "\"save\" <world_name>";
    let save_location;
    match params.pop_front() {
        Some(Literal::String(s)) => save_location = s,
        _ => return Err(anyhow!(help)),
    }
    let mut file = File::create(format!("{}/{}", WORLD_SAVE_FOLDER, save_location))?;

    // write seed
    file.write_all(&world.seed.to_le_bytes())?;

    let mut write_names = |names: Vec<String>| -> Result<()> {
        // write number of names
        file.write_all(&(names.len() as u32).to_le_bytes())?;
        // write strings, null terminated
        for name in names {
            file.write_all(name.as_bytes())?;
            file.write_all(&[0])?;
        }
        Ok(())
    };

    // write block names, then mob names
    let mut block_names = vec!["".to_string(); g.blocks.max_id as usize];
    for (k, v) in &g.blocks.id_to_name {
        block_names[*k as usize] = v.0.clone();
    }
    write_names(block_names)?;

    let mut mob_names = vec!["".to_string(); g.mob_templates.max_id.0 as usize];
    for (k, v) in &g.mob_templates.id_to_name {
        mob_names[k.0 as usize] = v.0.clone();
    }
    write_names(mob_names)?;

    // write world dimensions
    let dim = world.blocks().dim;
    file.write_all(&(dim.x() as u16).to_le_bytes())?;
    file.write_all(&(dim.y() as u16).to_le_bytes())?;
    file.write_all(&(dim.z() as u16).to_le_bytes())?;

    let size = dim.dim() as usize;

    // write blocks, then mobs, then colors
    let mut bytes = Vec::new();
    for i in 0..size {
        bytes.push(world.blocks().direct_get(i))
    }
    for i in 0..size {
        let data = world.mobs().direct_get(i).0.to_le_bytes();
        bytes.push(data[0]);
        bytes.push(data[1]);
    }
    for i in 0..size {
        let data = world.colors().direct_get(i);
        bytes.push(data.r);
        bytes.push(data.g);
        bytes.push(data.b);
    }

    file.write_all(&bytes)?;
    Ok(())
}

pub struct Load {
    pub seed: u64,
    pub block_names: Vec<String>,
    pub mob_names: Vec<String>,
    pub dim: Vector3,
    pub blocks: Vec<u8>,
    pub mobs: Vec<u16>,
    pub colors: Vec<RGB>,
}

fn load_world(name: &str) -> Result<Load> {
    let mut file = File::open(format!("{}/{}", WORLD_SAVE_FOLDER, name))?;

    // read the seed
    let mut buf = [0; 8];
    file.read_exact(&mut buf)?;
    let seed = u64::from_le_bytes(buf);

    let mut read_names = || -> Result<Vec<String>> {
        let mut buf = [0; 4];
        file.read_exact(&mut buf)?;
        let num = u32::from_le_bytes(buf);
        let mut names = Vec::new();

        for _ in 0..num {
            let mut string = Vec::new();
            // read a single null-terminated string,
            // read byte-by-byte till you see a 0
            loop {
                let mut val = [0; 1];
                file.read_exact(&mut val)?;
                let val = val[0];
                if val == 0 {
                    break;
                }
                string.push(val);
            }
            let string = String::from_utf8(string)?;
            names.push(string);
        }
        Ok(names)
    };

    // read block names and mob names
    let block_names = read_names()?;
    let mob_names = read_names()?;

    let mut read_u16 = || -> Result<u16> {
        let mut buf = [0; 2];
        file.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    };

    // read dimensions
    let dim = Vector3::new(
        read_u16()? as isize,
        read_u16()? as isize,
        read_u16()? as isize,
    );

    let size = dim.dim() as usize;

    // read blocks, mobs, and colors
    let mut blocks = vec![0; size];
    file.read_exact(&mut blocks)?;

    let mut mobs = vec![0; size * 2];
    file.read_exact(&mut mobs)?;
    let mobs = mobs
        .chunks_exact(2)
        .into_iter()
        .map(|a| u16::from_le_bytes([a[0], a[1]]))
        .collect();

    let mut colors = vec![0; size * 3];
    file.read_exact(&mut colors)?;
    let colors = colors
        .chunks_exact(3)
        .into_iter()
        .map(|a| RGB::new(a[0], a[1], a[2]))
        .collect();

    Ok(Load {
        seed,
        block_names,
        mob_names,
        dim,
        blocks,
        mobs,
        colors,
    })
}

type ServerCommand =
    dyn Fn(VecDeque<Literal>, &World, &Vec<Option<Player>>, &GameData) -> Result<()>;

fn init(args: &Vec<String>) -> Result<(GameData, World)> {
    fs::create_dir_all(WORLD_SAVE_FOLDER)?;
    fs::create_dir_all(PLAYER_SAVE_FOLDER)?;

    let m: GameMode = from_str(&fs::read_to_string("pvp/gamemode.jacl")?)?;

    let g;
    let load;
    if args[2] == "seed" {
        load = None;
        g = m.into_gamedata()?;
    } else if args[2] == "load" {
        let l = load_world(&args[3])?;
        g = m.into_gamedata_with_names(l.block_names.clone(), l.mob_names.clone())?;
        load = Some(l);
    } else {
        return Err(anyhow!("2nd argument must be 'seed' or 'load'"));
    }

    println!("read game data");

    let world = if let Some(load) = load {
        let w = World::from_load(load);
        println!("loaded world");
        w
    } else {
        println!("begin world generation...");
        let start = Instant::now();
        let w = World::from_seed(args[3].parse()?, &g);
        let duration = start.elapsed();
        println!("generated world in {:?}", duration);
        w
    }?;

    println!("ready to accept server commands:");
    Ok((g, world))
}

fn handle_server_commands(
    world: Arc<RwLock<World>>,
    players: Arc<RwLock<Vec<Option<Player>>>>,
    g: &GameData,
) -> Result<()> {
    let mut commands: HashMap<String, &ServerCommand> = HashMap::new();
    commands.insert("map".into(), &map);
    commands.insert("look".into(), &look);
    commands.insert("save".into(), &save_world);

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let val = format!("[{}]", line?);
        let mut params = VecDeque::from(from_str::<Vec<Literal>>(&val)?);
        let res = match params.pop_front() {
            Some(Literal::String(s)) => {
                if let Some(func) = commands.get(&s) {
                    let world = world.read().map_err(|_| anyhow!("couldn't lock world"))?;
                    let players = players
                        .read()
                        .map_err(|_| anyhow!("couldn't lock players"))?;
                    func(params, &world, &players, g)
                } else {
                    Err(anyhow!(format!(
                        "invalid command, choose one of the following {:?}",
                        commands.keys()
                    )))
                }
            }
            _ => Err(anyhow!("command must start with a string")),
        };
        match res {
            Err(e) => println!("{:?}", e),
            _ => {}
        }
    }
    Ok(())
}

enum ConnA {
    Init(Sender<(PlayerOut, Option<usize>)>),
    Run((VecDeque<Literal>, usize)),
    Quit(usize),
}

fn get_first_availible_id(players: &Vec<Option<Player>>) -> Option<usize> {
    for i in 0..players.len() {
        if players[i].is_none() {
            return Some(i);
        }
    }
    return None;
}
const LOCK_TEXT: &str = "couldn't aquire lock for";
fn world_op<T>(_: T) -> Error {
    anyhow!("{} world", LOCK_TEXT)
}
fn players_op<T>(_: T) -> Error {
    anyhow!("{} players", LOCK_TEXT)
}
fn battle_map_op<T>(_: T) -> Error {
    anyhow!("{} battle map", LOCK_TEXT)
}

fn world_logic(
    world_arc: Arc<RwLock<World>>,
    players_arc: Arc<RwLock<Vec<Option<Player>>>>,
    battle_map_arc: Arc<RwLock<BattleMap>>,
    g_arc: Arc<GameData>,
) -> Result<()> {
    let mut world = world_arc.write().map_err(world_op)?;
    let mut players = players_arc.write().map_err(players_op)?;
    let mut battle_map = battle_map_arc.write().map_err(battle_map_op)?;

    let dim = world.blocks().dim.clone();

    // start battles with mobs
    for i in 0..players.len() {
        if let Some(player) = &mut players[i] {
            if world.has_mob(*player.loc())? && battle_map.get_opponent(player.id()).is_err() {
                let mob = world.get_mob_at_mut(*player.loc(), &g_arc)?;
                battle_map.init_battle(Box::new(player), Box::new(mob), &g_arc)?;
                let mob_name = mob.name();
                player.send_text(format!("{}: {}\n", mob_name, mob.entrance()?));
                player.send_image(mob.display_img.clone());
            }
        }
    }

    let active_battles: Vec<BattleHandle> = battle_map.battles().cloned().collect();

    // handle pvp battles
    // just need to handle dying here,
    // players handle their own turns
    for battle_handle in &active_battles {
        let battle = battle_map.data_from_handle_mut(battle_handle).unwrap();
        let (a, b) = battle.ids();
        let (a, b) = match (a.enity_type, b.enity_type) {
            (EntityType::Player, EntityType::Player) => get_two_mut(a.id, b.id, &mut players)?,
            _ => continue,
        };
        let players = [a, b];
        for i in 0..players.len() {
            let curr = i;
            let other = (curr + 1) % 2;
            if players[curr].stats().health() <= 0.0 {
                let other_name = players[other].name();
                players[curr].send_text(format!("you were killed by {}\n", other_name));
                players[curr].send_text(format!("respawning...\n"));

                players[curr].stats_mut().reset_health(&g_arc);
                players[curr].stats_mut().reset_energy(&g_arc);
                players[curr].loc_mut().set(Vector3::new(
                    thread_rng().gen_range(0, dim.x()),
                    thread_rng().gen_range(0, dim.y()),
                    thread_rng().gen_range(0, dim.z()),
                ));

                let curr_name = players[curr].name();
                players[other].send_text(format!("you killed {}\n", curr_name));
                battle_map.end_battle(players[other].id())?;
            }
        }
    }

    // handle mob/player battles, including doing the mob's turn
    for battle_handle in &active_battles {
        let battle = battle_map.data_from_handle_mut(battle_handle)?;
        let (a, b) = battle.ids();
        let (entity, player) = match (a.enity_type, b.enity_type) {
            (EntityType::Mob, EntityType::Player) => {
                let mob = world.get_mob_mut(a.id)?;
                let opponent = get_mut(&mut players, b.id)?;
                (mob, opponent)
            }
            (EntityType::Player, EntityType::Mob) => {
                let mob = world.get_mob_mut(b.id)?;
                let opponent = get_mut(&mut players, a.id)?;
                (mob, opponent)
            }
            _ => {
                continue;
            }
        };
        let mob_health = entity.stats().health();
        let player_health = player.stats().health();

        if mob_health <= 0.0 || player_health <= 0.0 {
            let name = entity.name();
            let loss = entity.loss()?;
            let victory = entity.victory()?;

            if player_health <= 0.0 {
                player.send_text(format!("{}: {}\n", name, victory));
            }

            if mob_health <= 0.0 {
                player.send_text(format!("{}: {}\n", name, loss));
                player.send_text(format!("you killed {}\n", name));
                player.send_text("you got:\n".into());
                player.send_text(format!("{}\n", entity.drops().to_string()));
                player.inventory_mut().add_inventory(entity.drops());
                let loc = entity.loc().clone();
                world.delete_mob_by_loc(loc)?;
                battle_map.handle_status_effects(Box::new(player), &g_arc)?;
            }

            if player_health <= 0.0 {
                player.send_text(format!("you were killed by {}\n", name));
                player.send_text(format!("respawning...\n"));
                player.stats_mut().reset_health(&g_arc);
                player.stats_mut().reset_energy(&g_arc);
                player.loc_mut().set(Vector3::new(
                    thread_rng().gen_range(0, dim.x()),
                    thread_rng().gen_range(0, dim.y()),
                    thread_rng().gen_range(0, dim.z()),
                ));
                player.return_posn = *player.loc();
            }

            battle_map.end_battle(player.id())?;
            player.send_image("none".into());
            continue;
        }

        if !battle_map.turn(entity.id())? {
            continue;
        }

        player.send_text(format!("{}: {}\n", entity.name(), entity.attack()?));
        entity.do_random_move(Some(Box::new(player)), &mut battle_map, &g_arc);
        battle_map.do_turn(Box::new(entity), Box::new(player), &g_arc)?;
    }
    Ok(())
}

fn handle_player_input(
    player_input: ConnA,
    world_arc: Arc<RwLock<World>>,
    players_arc: Arc<RwLock<Vec<Option<Player>>>>,
    battle_map_arc: Arc<RwLock<BattleMap>>,
    g_arc: Arc<GameData>,
    rng: &mut StdRng,
) -> Result<()> {
    match player_input {
        ConnA::Init(s) => {
            let mut players = players_arc.write().map_err(players_op)?;
            let sender = s.clone();
            let id = get_first_availible_id(&players);
            if let Some(id) = id {
                let mut p_out = PlayerOut::new();
                p_out.add_pkt(g_arc.init_packet.clone());
                sender.send((p_out, Some(id)))?;
                let player = Player::new(id, sender, &g_arc, rng)?;

                players[id] = Some(player);
            } else {
                let mut p_out = PlayerOut::new();
                p_out.append_err(anyhow!("There are already 256 players in the game!"));
                sender.send((p_out, None))?;
            }
        }
        ConnA::Run((params, player_id)) => {
            let action_data = ActionData {
                params,
                player_id,
                world: world_arc.clone(),
                battle_map: battle_map_arc.clone(),
                players: players_arc.clone(),
                g: &g_arc,
            };
            let res = dispatch(action_data);
            if let Err(res) = res {
                let mut players = players_arc.write().map_err(players_op)?;
                let mut p_out = PlayerOut::new();
                p_out.append_err(res);
                players[player_id]
                    .as_mut()
                    .ok_or(anyhow!("bad player id"))?
                    .sender
                    .send((p_out, None))?;
            }
        }
        ConnA::Quit(player_id) => {
            let mut battle_map = battle_map_arc.write().map_err(battle_map_op)?;
            let mut players = players_arc.write().map_err(players_op)?;
            let mut world = world_arc.write().map_err(world_op)?;
            if let Ok(opponent) = battle_map.get_opponent(ID::player(player_id)) {
                let entity = get_entity(opponent, &mut players, &mut world)?;
                entity.send_text("your opponent disconnected!\n".into());
                battle_map.end_battle(opponent)?;
            }
            players[player_id] = None;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // init gamedata and world
    let (g, world) = init(&args)?;

    let (send, recv) = unbounded();

    let mut players = vec![];
    for _ in 0..(std::u8::MAX as usize + 1) {
        players.push(None);
    }

    let battle_map = BattleMap::new();
    let g = Arc::new(g);
    let world = Arc::new(RwLock::new(world));
    let players = Arc::new(RwLock::new(players));
    let battle_map = Arc::new(RwLock::new(battle_map));

    let g_arc = Arc::clone(&g);
    let world_arc = Arc::clone(&world);
    let players_arc = Arc::clone(&players);
    let battle_map_arc = Arc::clone(&battle_map);

    // handle player input and world logic
    // world logic runs whenever player input isn't happening
    spawn(move || {
        let mut rng = SeedableRng::seed_from_u64(thread_rng().gen());
        loop {
            let res = recv.try_recv();
            let res = match res {
                Ok(player_input) => {
                    // we have input from a player, handle it
                    handle_player_input(
                        player_input,
                        world_arc.clone(),
                        players_arc.clone(),
                        battle_map_arc.clone(),
                        g_arc.clone(),
                        &mut rng,
                    )
                }
                Err(_) => {
                    // no player input to handle, just do world logic
                    world_logic(
                        world_arc.clone(),
                        players_arc.clone(),
                        battle_map_arc.clone(),
                        g_arc.clone(),
                    )
                }
            };
            // if we encounter any errors, print them
            if let Err(err) = res {
                println!("{}", err);
            }
        }
    });

    // spin up new thread for each connection we get
    let port: u64 = args[1].parse()?;
    let server = Server::bind(format!("0.0.0.0:{}", port))?;
    spawn(move || {
        for request in server.filter_map(Result::ok) {
            let send = send.clone();
            spawn(move || {
                let client = request.accept().unwrap();
                let ip = client.peer_addr().unwrap();
                println!("Connection from {}", ip);
                handle_connection(client, send);
            });
        }
    });

    // autosave for players
    let players_clone = players.clone();
    spawn(move || {
        let players = players_clone;
        loop {
            let dur = Duration::from_secs(10);
            thread::sleep(dur);

            let players = players.read().unwrap();
            for player in players.iter() {
                if let Some(player) = player {
                    if let Some(username) = &player.username {
                        let save_file = format!("{}/{}", PLAYER_SAVE_FOLDER, username);
                        let mut file = OpenOptions::new().write(true).truncate(true).open(save_file).unwrap();
                        file.write_all(&player.save().unwrap().as_bytes()).unwrap();
                    }
                }
            }
        }
    });

    // handle server commands on the console
    // print out any errors
    loop {
        if let Err(res) = handle_server_commands(Arc::clone(&world), Arc::clone(&players), &g) {
            println!("{:?}\n", res);
        }
    }
}

fn handle_connection(stream: Client<TcpStream>, channel: Sender<ConnA>) {
    let (send, recv) = unbounded();
    while let Err(e) = channel.send(ConnA::Init(send.clone())) {
        println!("{}", e);
    }
    let mut res = recv.recv().unwrap();
    let id = res.1.unwrap();
    let init_pkt = res.0.get_pkt().unwrap();

    let (mut reader, mut writer) = stream.split().unwrap();
    writer
        .send_message(&OwnedMessage::Binary(init_pkt.bytes()))
        .unwrap();

    let (quit_tx, quit_rx) = unbounded();
    spawn(move || loop {
        if let Ok(_) = quit_rx.try_recv() {
            break;
        }
        if let Ok((mut res, _)) = recv.try_recv() {
            while let Some(pkt) = res.get_pkt() {
                let message = OwnedMessage::Binary(pkt.bytes());
                if writer.send_message(&message).is_err() {
                    break;
                }
            }
        }
    });

    loop {
        let line;
        if let Ok(l) = reader.recv_message() {
            line = l;
        } else {
            break;
        }

        let text;
        if let OwnedMessage::Text(l) = line {
            text = l;
        } else {
            break;
        }

        let params;
        if let Ok(p) = from_str::<Vec<Literal>>(&text) {
            params = p;
        } else {
            continue; // TODO: should replace this with an error message
        }

        channel
            .send(ConnA::Run((VecDeque::from_iter(params), id)))
            .unwrap();
    }

    quit_tx.send(()).unwrap();
    channel.send(ConnA::Quit(id)).unwrap();
}

pub fn get_entity<'a>(
    entity: ID,
    players: &'a mut Vec<Option<Player>>,
    world: &'a mut World,
) -> Result<Box<&'a mut dyn Entity>> {
    match entity.enity_type {
        EntityType::Mob => {
            let entity = world.get_mob_mut(entity.id)?;
            Ok(Box::new(entity as &mut dyn Entity))
        }
        EntityType::Player => {
            let entity = get_mut(players, entity.id)?;
            Ok(Box::new(entity as &mut dyn Entity))
        }
    }
}
