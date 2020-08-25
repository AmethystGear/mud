extern crate ansi_term;
extern crate rstring_builder;

use crate::action::Action;
use crate::entities::SpawnedEntities;
use crate::player::Player;
use crate::playerout::PlayerOut;
use crate::scanner::Param;
use std::net::TcpStream;
use std::sync::mpsc;
use std::sync::{
    mpsc::{Receiver, Sender},
    Arc, Mutex,
};
use std::thread::{self, spawn};
use std::u8;

use action::ActionMap;
use std::env;
use std::{
    error::Error,
    fs::{self, File, OpenOptions},
    time::{Duration, SystemTime},
};

use websocket::{sync::Client, sync::Server, OwnedMessage};

mod action;
mod display;
mod entities;
mod perlin_noise;
mod player;
mod playerout;
mod scanner;
mod stats;
mod world;

type ConnOut = (
    Option<String>,
    Option<Vec<Param>>,
    Option<Action>,
    Option<Sender<(PlayerOut, Option<u8>)>>,
    Option<u8>,
);
type ConnIn = (PlayerOut, Option<u8>);

const SAVE: &str = "save";
const WORLD_SAVE: &str = "save/world_save";
const PVP_TERRAIN_CONFIG: &str = "config/terrain/pvp";
const PVE_TERRAIN_CONFIG: &str = "config/terrain/pve";
const PLAYER_AUTOSAVE_INTERVAL: u64 = 5;
const WORLD_AUTOSAVE_INTERVAL: u64 = 240;

fn main() {
    let args: Vec<String> = env::args().collect();
    let port: u64 = args[1].parse().unwrap();
    let server = Server::bind(format!("0.0.0.0:{}", port)).unwrap();
    let (send, recv): (Sender<ConnOut>, Receiver<ConnOut>) = mpsc::channel();

    let world;
    if args[2] == "load" {
        let file = File::open(WORLD_SAVE).unwrap();
        world = world::from_save(file);
    } else {
        let seed: i64 = args[2].parse().unwrap();
        let file;
        if args[3] == "pvp" {
            file = File::open(PVP_TERRAIN_CONFIG).unwrap();
        } else if args[3] == "pve" {
            file = File::open(PVE_TERRAIN_CONFIG).unwrap();
        } else {
            panic!("expected 3rd argument to be pvp or pve!")
        }
        world = world::from_seed(seed, file, args[3].clone());
    }
    if world.is_err() {
        panic!("{}", world.err().unwrap());
    }
    let world = Arc::new(Mutex::new(world.unwrap()));
    let world_clone = world.clone();

    let mut players = vec![]; // max cap of 256 players per server.
    for _ in 0..(std::u8::MAX as usize + 1) {
        players.push(None);
    }
    let players = Arc::new(Mutex::new(players));
    let players_clone = players.clone();
    spawn(move || {
        println!("generated world");

        let mut spawned_entities = SpawnedEntities::new();

        let action_map = action::get_action_map();

        loop {
            let res = recv.try_recv();
            // as long as there is player input to process, handle that first
            if let Ok(res) = res {
                let mut world = world.lock().unwrap();
                let mut players = players.lock().unwrap();
                handle_player_inp(
                    res,
                    &mut players,
                    &mut world,
                    &mut spawned_entities,
                    &action_map,
                );
            } else {
                // then handle any background game logic as needed.
                // this adds time limits to pvp turns.
                let mut players = players.lock().unwrap();
                let mut opponents = vec![];
                for player in &mut *players {
                    if let Some(player) = player {
                        if player::turn(player) {
                            if let Some(opponent) = player.opponent() {
                                if SystemTime::now()
                                    .duration_since(player.get_last_turn_time())
                                    .expect("time went backwards??")
                                    .as_millis()
                                    > player::MAX_TURN_TIME_MILLIS
                                {
                                    opponents.push(opponent);
                                    player::set_turn(player, false);
                                    player::send_str(
                                        player,
                                        "You're TOO SLOW!!! Your turn is up!\n",
                                    )
                                    .unwrap();
                                }
                            }
                        }
                    }
                }
                for opponent in opponents {
                    let opp = players[opponent as usize].as_mut().unwrap();
                    player::set_turn(opp, true);
                    opp.set_last_turn_time(SystemTime::now());
                    player::send_str(opp, "Your opponent was TOO SLOW!!! It's your turn now!\n")
                        .unwrap();
                }
            }
        }
    });
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
        println!("exiting")
    });
    let players_world_save = players_clone.clone();

    // only autosave in pve
    if args[3] == "pve" {
        // player autosave loop
        spawn(move || loop {
            {
                let players = &mut *players_clone.lock().unwrap();
                for player in players {
                    let player = player.as_mut();
                    if let Some(player) = player {
                        let player_name = get_player_name(player).unwrap();
                        let mut save = OpenOptions::new()
                            .write(true)
                            .create(true)
                            .open(format!("{}/{}", SAVE, player_name))
                            .unwrap();
                        player::save_to(player, &mut save).unwrap();
                    }
                }
            }
            thread::sleep(Duration::from_secs(PLAYER_AUTOSAVE_INTERVAL));
        });
        // world autosave loop
        spawn(move || loop {
            {
                let players = &mut *players_world_save.lock().unwrap();
                for player in players {
                    let player = player.as_mut();
                    if let Some(player) = player {
                        if player::send_str(player, "autosaving world...\n").is_err() {
                            println!("couldn't send world autosave message!");
                        }
                    }
                }
            }
            {
                let w = world_clone.lock().unwrap();
                let mut save = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(WORLD_SAVE)
                    .unwrap();
                fs::create_dir_all(SAVE).unwrap();
                world::save_to(&w, &mut save).unwrap();
            }
            {
                let players = &mut *players_world_save.lock().unwrap();
                for player in players {
                    let player = player.as_mut();
                    if let Some(player) = player {
                        if player::send_str(player, "world saved!\n").is_err() {
                            println!("couldn't send world autosave message!");
                        }
                    }
                }
            }
            thread::sleep(Duration::from_secs(WORLD_AUTOSAVE_INTERVAL));
        });
    }

    // this is to ensure none of our threads just quit
    loop {}
}

fn handle_connection(stream: Client<TcpStream>, channel: Sender<ConnOut>) {
    let action_map = action::get_action_map();
    let (send, recv): (Sender<ConnIn>, Receiver<ConnIn>) = mpsc::channel();
    let id;

    // initialization step
    while let Err(e) = channel.send((None, None, None, Some(send.clone()), None)) {
        println!("{}", e);
    }
    let mut res = recv.recv().unwrap();
    while res.1.is_none() {
        while let Err(e) = channel.send((None, None, None, Some(send.clone()), None)) {
            println!("{}", e);
        }
        res = recv.recv().unwrap();
    }
    id = res.1.unwrap();
    let pkt = res.0.get_pkt();

    let (mut reader, mut writer) = stream.split().unwrap();

    writer.send_message(&OwnedMessage::Binary(pkt.unwrap().bytes())).unwrap();

    spawn(move || loop {
        let (mut response, _) = recv.recv().unwrap();
        let mut pkt = response.get_pkt();
        while pkt.is_some() {
            if writer.send_message(&OwnedMessage::Binary(pkt.unwrap().bytes())).is_err() {
                writer.shutdown_all().unwrap();
                return;
            }
            pkt = response.get_pkt();
        }
    });

    let mut last_res: Option<(String, Vec<Param>, Action)> = None;
    loop {
        let line = reader.recv_message();
        
        if line.is_err() {
            break;
        }

        let l;
        if let OwnedMessage::Text(line) = line.unwrap() {
            l = line;
        } else {
            break;
        }
        let line = l;
        println!("{}", line);

        let action_res: Result<(String, Vec<Param>, Action), Box<dyn Error>>;
        if line.trim() == "" {
            let clone = last_res.clone();
            if clone.is_some() {
                action_res = Ok(clone.unwrap());
            } else {
                action_res = Err("no last successful command to run!".into());
            }
        } else {
            action_res = action::get_action_and_params(&action_map, line.clone());
            let res = action::get_action_and_params(&action_map, line);
            if res.is_ok() {
                last_res = Some(res.ok().unwrap());
            } else {
                last_res = None;
            }
        }
        if action_res.is_ok() {
            let (keyword, params, action) = action_res.unwrap();
            let res = channel.send((Some(keyword), Some(params), Some(action), None, Some(id)));
            if res.is_err() {
                println!("end conn");
                break;
            }
        }
    }
}

fn get_first_availible_id(players: &Vec<Option<Player>>) -> Option<u8> {
    for i in 0..players.len() {
        if players[i].is_none() {
            return Some(i as u8);
        }
    }
    return None;
}

fn get_player_name(player: &Player) -> Result<String, Box<dyn Error>> {
    return stats::get(&stats::get(player.data(), "identity")?.as_box()?, "name")?.as_string();
}

fn handle_player_inp(
    data: ConnOut,
    players: &mut Vec<Option<Player>>,
    world: &mut world::World,
    spawned_entities: &mut SpawnedEntities,
    action_map: &ActionMap,
) {
    let (keyword, params, action, sender, id) = data;
    let player_id;
    if sender.is_some() {
        let sender = sender.clone().unwrap();
        let id = get_first_availible_id(&players);
        if id.is_none() {
            let mut p_out = PlayerOut::new();
            let err: Result<u8, Box<dyn Error>> =
                Err("There are already 256 players in the game!".into());
            p_out.append_err(err.err().unwrap());
            sender.send((p_out, None)).unwrap();
            return;
        }
        let player = player::from(0, 0, id.unwrap(), sender.clone());
        if player.is_err() {
            println!("failed to create player!");
            return;
        }
        let mut player =
            player.expect("just checked that player is not err, so this should never fail");
        let res = player::respawn(&mut player, &world);
        if res.is_err() {
            println!("failed to respawn player!");
            return;
        }

        player_id = id.unwrap();
        players[player_id as usize] = Some(player);
        let mut p_out = PlayerOut::new();
        p_out.add_pkt(playerout::get_init(&world).unwrap());
        sender.send((p_out, Some(player_id))).unwrap();
        return;
    } else if id.is_some() {
        player_id = id.unwrap();
    } else {
        unreachable!("both id and sender are None!");
    }
    let keyword = keyword.expect("if id is not None, then keyword should be Some");
    let params = params.expect("if params is not None, then params should be Some");
    let action = action.expect("if id is not none, then action should be Some");

    let x;
    let y;
    let mut res;
    {
        let result = action.run(
            Some(spawned_entities),
            Some(&action_map),
            Some(keyword),
            Some(&params),
            Some(player_id),
            Some(players),
            Some(world),
        );
        if result.is_none() {
            println!("bad params to function");
            return;
        }
        let result = result.unwrap();
        let player = players[player_id as usize].as_ref();
        if player.is_none() {
            return;
        }
        let player = player.unwrap();
        let x_ = player::x(&player);
        let y_ = player::y(&player);
        if x_.is_err() || y_.is_err() {
            return;
        }
        x = x_.unwrap();
        y = y_.unwrap();
        if !entities::has_entity(&spawned_entities, x, y) && world::has_entity(&world, x, y) {
            let name = world::get_entity_name(&world, x, y).unwrap();
            let stats = world::get_entity_properties(&world, x, y).unwrap().clone();
            let err = entities::spawn(stats, x, y, name.clone(), spawned_entities, world);
            if err.is_err() {
                println!("{}", name.clone());
                println!("{}", err.err().unwrap().to_string());
            }
        }
        match result {
            Ok(ok) => {
                res = ok;
            }
            Err(err) => {
                res = PlayerOut::new();
                res.append_err(err);
            }
        }
    }
    let mut mob_action_res = None;
    let interact;
    {
        let player = players[player_id as usize].as_ref().unwrap();
        interact = player.interact();
    }
    if entities::has_entity(&spawned_entities, x, y) && !interact {
        let entity_action: Action =
            entities::get_entity_action(spawned_entities, "interact".to_string(), x, y).unwrap();
        let result = entity_action.run(
            Some(spawned_entities),
            None,
            None,
            None,
            Some(player_id),
            Some(players),
            Some(world),
        );
        if result.is_none() {
            return;
        }
        let result = result.unwrap();
        if result.is_ok() {
            mob_action_res = Some(result.ok().unwrap());
        }
        let player = players[player_id as usize].as_mut().unwrap();
        player.set_interact(true);
    }

    match mob_action_res {
        Some(some) => {
            res.append_player_out(some);
        }
        None => {}
    }

    match players[player_id as usize].as_ref() {
        Some(player) => {
            let res = player::send(player, res);
            if res.is_err() {
                players[player_id as usize] = None;
            }
        }
        None => println!("Invalid player id {}", player_id),
    }
}
