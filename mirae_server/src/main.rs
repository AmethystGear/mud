extern crate ansi_term;
extern crate rstring_builder;

use crate::action::Action;
use crate::entities::SpawnedEntities;
use crate::player::Player;
use crate::playerout::PlayerOut;
use crate::scanner::Param;
use std::io::Write;
use std::io::{self, BufRead, BufReader, BufWriter};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::sync::{
    mpsc::{Receiver, Sender},
    Arc, Mutex,
};
use std::thread::spawn;
use std::u8;

use action::ActionMap;
use std::env;
use std::{error::Error, fs::File, time::SystemTime};

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

const WORLD_SAVE: &str = "world_save";
const DEFAULT_PORT: u64 = 31415;

fn handle_connection(stream: TcpStream, channel: Sender<ConnOut>) {
    let action_map = action::get_action_map();
    let (send, recv): (Sender<ConnIn>, Receiver<ConnIn>) = mpsc::channel();
    let id;
    let stream_clone = stream.try_clone().unwrap();
    let mut reader = BufReader::new(stream_clone);
    let stream_clone = stream.try_clone().unwrap();
    let mut writer = BufWriter::new(stream_clone);

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
        writer.flush().unwrap();
    }
    id = res.1.unwrap();
    let pkt = res.0.get_pkt();
    writer.write_all(&pkt.unwrap().bytes()).unwrap();
    writer.flush().unwrap();

    let (s, r) = mpsc::channel();
    // packet send step
    spawn(move || loop {
        let (mut response, _) = recv.recv().unwrap();
        let mut pkt = response.get_pkt();
        while pkt.is_some() {
            if (writer.write_all(&pkt.unwrap().bytes())).is_err() {
                s.send(true).unwrap();
                return;
            }
            pkt = response.get_pkt();
        }
        let res = writer.flush();
        if res.is_err() {
            s.send(true).unwrap();
            return;
        }
    });

    let mut last_res: Option<(String, Vec<Param>, Action)> = None;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).is_err() {
            println!("end conn");
            break;
        }
        if let Ok(res) = r.try_recv() {
            if res {
                break;
            }
        }

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

fn main() {
    let args: Vec<String> = env::args().collect();
    let port: u64 = args[1].parse().unwrap();
    let server = TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap();
    let (send, recv): (Sender<ConnOut>, Receiver<ConnOut>) = mpsc::channel();

    let world;
    if args[2] == "load" {
        let file = File::open(WORLD_SAVE).unwrap();
        world = world::from_save(file);
    } else {
        let seed: i64 = args[2].parse().unwrap();
        world = world::from_seed(seed);
    }
    if world.is_err() {
        panic!("{}", world.err().unwrap());
    }
    let world = Arc::new(Mutex::new(world.unwrap()));
    let world_clone = world.clone();
    spawn(move || {
        println!("generated world");

        let mut spawned_entities = SpawnedEntities::new();

        let action_map = action::get_action_map();
        let mut players: Vec<Option<Player>> = Vec::new(); // max cap of 256 players per server.
        for _ in 0..(std::u8::MAX as usize + 1) {
            players.push(None);
        }

        loop {
            let res = recv.try_recv();
            // as long as there is player input to process, handle that first
            if let Ok(res) = res {
                let mut world = world.lock().unwrap();
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
                let mut opponents = vec![];
                for player in &mut players {
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
        for stream in server.incoming() {
            match stream {
                Err(_) => println!("listen error"),
                Ok(stream) => {
                    println!(
                        "connection from {} to {}",
                        stream.peer_addr().unwrap(),
                        stream.local_addr().unwrap()
                    );
                    let send = send.clone();
                    spawn(move || {
                        handle_connection(stream, send);
                    });
                }
            }
        }
    });
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        if let Ok(line) = line {
            if line == "save" {
                let w = world_clone.lock().unwrap();
                let save = File::open(WORLD_SAVE).unwrap();
                world::save_to(&w, save).unwrap();
            } else {
                println!("did not recognize command!")
            }
        } else {
            println!("could not read from stdin!");
        }
    }
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
