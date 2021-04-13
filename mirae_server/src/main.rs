#![allow(dead_code)]
use actions::{dispatch, get_mut, ActionData};
use anyhow::{anyhow, Result};
use combat::{BattleHandle, BattleMap, EntityType, ID};
use display::{Bounds, Image};
use entity::Entity;
use gamedata::gamedata::{GameData, GameMode};
use image::{ImageBuffer, Rgb};
use io::BufRead;
use mpsc::{Sender, TryRecvError};
use player::Player;
use playerout::PlayerOut;
use rand::{thread_rng, Rng};
use serde_jacl::{
    de::from_str,
    structs::{Literal, Number},
};
use std::{
    collections::{HashMap, VecDeque},
    env, fs, io,
    iter::FromIterator,
    net::TcpStream,
    sync::{mpsc, Arc, Mutex},
    thread::spawn,
    time::Instant,
};
use vector3::Vector3;
use websocket::{
    sync::{Client, Server},
    OwnedMessage,
};
use world::World;

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

fn save_world(_params: VecDeque<Literal>, _world: &World, _g: &GameData) {}

type ServerCommand =
    dyn Fn(VecDeque<Literal>, &World, &Vec<Option<Player>>, &GameData) -> Result<()>;

fn init() -> Result<(GameData, World)> {
    let m: GameMode = from_str(&fs::read_to_string("pvp/gamemode.jacl")?)?;
    let g = m.into_gamedata()?;
    println!("read game data");
    println!("begin world generation...");
    let start = Instant::now();
    let world = World::from_seed(thread_rng().gen(), &g)?;
    let duration = start.elapsed();
    println!("generated world in {:?}", duration);
    println!("world dimensions: {:?}", world.blocks().dim);
    Ok((g, world))
}

fn handle_server_commands(
    world: Arc<Mutex<World>>,
    players: Arc<Mutex<Vec<Option<Player>>>>,
    g: &GameData,
) -> Result<()> {
    let mut commands: HashMap<String, &ServerCommand> = HashMap::new();
    commands.insert("map".into(), &map);
    commands.insert("look".into(), &look);

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let val = format!("[{}]", line?);
        let mut params = VecDeque::from(from_str::<Vec<Literal>>(&val)?);
        let res = match params.pop_front() {
            Some(Literal::String(s)) => {
                if let Some(func) = commands.get(&s) {
                    let world = world.lock().map_err(|_| anyhow!("couldn't lock world"))?;
                    let players = players
                        .lock()
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

fn main() -> Result<()> {
    // init gamedata and world
    let (g, world) = init()?;

    let (send, recv) = mpsc::channel();

    let mut players = vec![];
    for _ in 0..(std::u8::MAX as usize + 1) {
        players.push(None);
    }

    let battle_map = BattleMap::new();
    let g = Arc::new(g);
    let world = Arc::new(Mutex::new(world));
    let players = Arc::new(Mutex::new(players));
    let battle_map = Arc::new(Mutex::new(battle_map));

    let g_arc = Arc::clone(&g);
    let world_arc = Arc::clone(&world);
    let players_arc = Arc::clone(&players);
    let battle_map_arc = Arc::clone(&battle_map);
    spawn(move || {
        loop {
            let res: Result<ConnA, TryRecvError> = recv.try_recv();
            match res {
                Ok(res) => {
                    // we have input from a player, handle it
                    let mut world = world_arc.lock().unwrap();
                    let mut players = players_arc.lock().unwrap();
                    let mut battle_map = battle_map_arc.lock().unwrap();
                    match res {
                        ConnA::Init(s) => {
                            let sender = s.clone();
                            let id = get_first_availible_id(&players);
                            if let Some(id) = id {
                                let mut p_out = PlayerOut::new();
                                p_out.add_pkt(g_arc.init_packet.clone());
                                sender.send((p_out, Some(id))).unwrap();
                                let player =
                                    Player::new(id, sender, &g_arc, &mut world.rng).unwrap();

                                players[id] = Some(player);
                            } else {
                                let mut p_out = PlayerOut::new();
                                p_out.append_err(anyhow!(
                                    "There are already 256 players in the game!"
                                ));
                                sender.send((p_out, None)).unwrap();
                            }
                        }
                        ConnA::Run((params, player_id)) => {
                            let action_data = ActionData {
                                params,
                                player_id,
                                world: &mut world,
                                g: &g_arc,
                                battle_map: &mut battle_map,
                                players: &mut players,
                            };
                            let res = dispatch(action_data);
                            if let Err(res) = res {
                                let mut p_out = PlayerOut::new();
                                p_out.append_err(res);
                                players[player_id]
                                    .as_mut()
                                    .unwrap()
                                    .sender
                                    .send((p_out, None))
                                    .unwrap();
                            }
                        }
                        ConnA::Quit(player_id) => {
                            if let Ok(opponent) = battle_map.get_opponent(ID::player(player_id)) {
                                let entity =
                                    get_entity(opponent, &mut players, &mut world).unwrap();
                                entity.send_text("your opponent disconnected!\n".into());
                                battle_map.end_battle(opponent).unwrap();
                            }
                            players[player_id] = None;
                        }
                    }
                }
                Err(_) => {
                    /*no input to handle, do any world logic */
                    let mut world = world_arc.lock().unwrap();
                    let mut players = players_arc.lock().unwrap();
                    let mut battle_map = battle_map_arc.lock().unwrap();

                    let active_battles: Vec<BattleHandle> = battle_map.battles().cloned().collect();
                    for battle_handle in &active_battles {
                        let dim = world.blocks().dim.clone();
                        let battle = battle_map.data_from_handle_mut(battle_handle).unwrap();
                        let (a, b) = battle.ids();
                        let (mob, player) = match (a.enity_type, b.enity_type) {
                            (EntityType::Mob, EntityType::Player) => {
                                let mob = world.get_mob_mut(a.id).unwrap();
                                let opponent = get_mut(&mut players, b.id).unwrap();
                                (mob, opponent)
                            }
                            (EntityType::Player, EntityType::Mob) => {
                                let mob = world.get_mob_mut(b.id).unwrap();
                                let opponent = get_mut(&mut players, a.id).unwrap();
                                (mob, opponent)
                            }
                            _ => {
                                continue;
                            }
                        };

                        let player_health = player.stats().health();
                        let mob_health = mob.stats().health();

                        if player_health <= 0.0 {
                            player.send_text(format!(
                                "{}: {}\n",
                                mob.name().unwrap(),
                                mob.victory().unwrap()
                            ));
                            player
                                .send_text(format!("you were killed by {}\n", mob.name().unwrap()));
                            player.send_text(format!("respawning...\n"));
                            player.stats_mut().reset_health(&g_arc);
                            player.stats_mut().reset_energy(&g_arc);
                            player.loc_mut().set(Vector3::new(
                                thread_rng().gen_range(0, dim.x()),
                                thread_rng().gen_range(0, dim.y()),
                                thread_rng().gen_range(0, dim.z()),
                            ));
                            battle_map.end_battle(player.id()).unwrap();
                            player.send_image("none".into());
                            continue;
                        }

                        if mob_health <= 0.0 {
                            player.send_text(format!(
                                "{}: {}\n",
                                mob.name().unwrap(),
                                mob.loss().unwrap()
                            ));
                            player.send_text(format!("you killed {}\n", mob.name().unwrap()));
                            player.send_text("you got:\n".into());
                            player.send_text(format!("{}\n", mob.drops().to_string()));
                            player.inventory_mut().add_inventory(mob.drops());
                            let loc = mob.loc().clone();
                            world.delete_mob_by_loc(loc).unwrap();
                            battle_map.end_battle(player.id()).unwrap();
                            player.send_image("none".into());
                            continue;
                        }

                        if !battle_map.turn(mob.id()).unwrap() {
                            continue;
                        }

                        player.send_text(format!(
                            "{}: {}\n",
                            mob.name().unwrap(),
                            mob.attack().unwrap()
                        ));

                        mob.do_random_move(Some(Box::new(player)), &mut battle_map, &g_arc);
                        battle_map
                            .do_turn(Box::new(mob), Box::new(player), &g_arc)
                            .unwrap();
                    }
                }
            }
        }
    });

    let args: Vec<String> = env::args().collect();
    let port: u64 = args[1].parse().unwrap();
    let server = Server::bind(format!("0.0.0.0:{}", port)).unwrap();

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

    // handle server commands on the console
    // print out any errors
    loop {
        if let Err(res) = handle_server_commands(Arc::clone(&world), Arc::clone(&players), &g) {
            println!("{:?}\n", res);
        }
    }
}

fn handle_connection(stream: Client<TcpStream>, channel: Sender<ConnA>) {
    let (send, recv) = mpsc::channel();
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

    let (quit_tx, quit_rx) = mpsc::channel();
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
