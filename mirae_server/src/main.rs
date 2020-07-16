extern crate ansi_term;

use crate::scanner::Param;
use crate::action::Action;
use crate::player::Player;
use crate::entities::SpawnedEntities;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::thread::spawn;
use std::io::{BufReader, BufRead, BufWriter};
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::u8;

mod scanner;
mod stats;
mod action;
mod world;
mod perlin_noise;
mod player;
mod display;
mod entities;

fn init(reader : &mut BufReader<TcpStream>, action_map : &action::ActionMap, send: &Sender<(String, Option<u8>)>, recv : &Receiver<(String, Option<u8>)>, channel : &Sender<(String, Vec<Param>, Action, Option<Sender<(String, Option<u8>)>>, Option<u8>)>) -> (String, Option<u8>) {
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    let action_res = action::get_action_and_params(action_map, line);
    if action_res.is_ok() {
        let (keyword, params, action) = action_res.unwrap();
        let res = channel.send((keyword, params, action, Some(send.clone()), None));
        if res.is_err() {
            println!("\n\nERROR: {}\n\n", res.err().unwrap());
        }
        return recv.recv().unwrap();
    } else {
        let err = ansi_term::Color::Red.paint(action_res.err().unwrap());
        return (err.to_string(), None);
    }
}

fn handle_connection(stream: TcpStream, channel : Sender<(String, Vec<Param>, Action, Option<Sender<(String, Option<u8>)>>, Option<u8>)>) {
    let action_map = action::get_action_map();
    let (send, recv) : (Sender<(String, Option<u8>)>, Receiver<(String, Option<u8>)>) = mpsc::channel();
    let id;
    let stream_clone = stream.try_clone().unwrap();
    let mut reader = BufReader::new(stream_clone);
    let stream_clone = stream.try_clone().unwrap();
    let mut writer = BufWriter::new(stream_clone);

    let mut res = init(&mut reader, &action_map, &send, &recv, &channel);
    while res.1.is_none() {
        res = init(&mut reader, &action_map, &send, &recv, &channel);
        writer.write_all(format!("/begin/{}/end/\n", res.0).as_bytes()).unwrap();
        writer.flush().unwrap();
    }
    id = res.1.unwrap();
    writer.write_all(format!("/begin/{}/end/\n", res.0).as_bytes()).unwrap();
    writer.flush().unwrap();

    spawn(move || {
        loop {
            let (string_response, _) = recv.recv().unwrap();
            let string_response = format!("/begin/{}/end/\n", string_response);
            writer.write_all(string_response.as_bytes()).unwrap();
            writer.flush().unwrap();
        }
    });

    loop {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let action_res = action::get_action_and_params(&action_map, line);
        if action_res.is_ok() {
            let (keyword, params, action) = action_res.unwrap();
            let res = channel.send((keyword, params, action, None, Some(id)));
            if res.is_err() {
                println!("\n\nERROR: {}\n\n", res.err().unwrap());
            }
        } else {
            let err =  ansi_term::Color::Red.paint(action_res.err().unwrap());
            send.send((err.to_string(), None)).unwrap();
        }
    }
}


fn get_first_availible_id(players : &Vec<Option<Player>>) -> Option<u8> {
    for i in 0..players.len() {
        if players[i].is_none() {
            return Some(i as u8);
        }
    }
    return None;
}

fn main() {
    let server = TcpListener::bind("0.0.0.0:31415").unwrap();
    let (send, recv) : (Sender<(String, Vec<Param>, Action, Option<Sender<(String, Option<u8>)>>, Option<u8>)>,
                        Receiver<(String, Vec<Param>, Action, Option<Sender<(String, Option<u8>)>>, Option<u8>)>) = mpsc::channel();

    spawn(move || {
        let mut world = world::from_seed(0);
        let mut spawned_entities = SpawnedEntities::new();

        let action_map = action::get_action_map();
        let mut players : Vec<Option<Player>> = Vec::new(); // max cap of 256 players per server.
        for _ in 0..(std::u8::MAX as usize + 1) {
            players.push(None);
        }

        loop {
            let (keyword, params, action, sender, id) = recv.recv().unwrap();

            let player_id;
            if sender.is_some() {
                let sender = sender.clone().unwrap();
                let id = get_first_availible_id(&players);
                if id.is_none() {
                    sender.send(("Cannot enter game! There are already 256 players on the server!".to_string(), None)).unwrap();
                    continue;
                }
                let mut player = player::from(0, 0, id.unwrap(), sender.clone());
                player::respawn(&mut player, &world);
                println!("{}", id.unwrap());
                player_id = id.unwrap();
                players[player_id as usize] = Some(player);
            } else if id.is_some() {
                player_id = id.unwrap();
            } else {
                unreachable!("both id and sender are None!");
            }

            let x;
            let y;
            let mut string_result;
            {
                let result = action.run(Some(&mut spawned_entities), Some(&action_map), Some(keyword), Some(&params),
                                        Some(player_id), Some(&mut players), Some(&mut world));
                let player = players[player_id as usize].as_ref().unwrap();
                x = player::x(&player);
                y = player::y(&player);
                if !entities::has_entity(&spawned_entities, x, y) && world::has_entity(&world, x, y) {
                    let name = world::get_entity_name(&world, x, y).unwrap();
                    let stats = world::get_entity_properties(&world, x, y).unwrap().clone();
                    entities::spawn(stats, x, y, name, &mut spawned_entities, &mut world);
                }
                if result.is_ok() {
                    string_result = result.ok().unwrap().string();
                } else {
                    let err = ansi_term::Color::Red.paint(result.err().unwrap()).to_string();
                    string_result = format!("{}\n", err);
                }
            }
            let mut mob_action_res = None;
            let interact;
            {
                let player = players[player_id as usize].as_ref().unwrap();
                interact = player.interact();
            }
            if entities::has_entity(&spawned_entities, x, y) && !interact {
                let entity_action : Action = entities::get_entity_action(&mut spawned_entities, "interact".to_string(), x, y).unwrap();
                let result = entity_action.run(Some(&mut spawned_entities), None, None, None, Some(player_id), Some(&mut players), Some(&mut world));
                if result.is_ok() {
                    mob_action_res = Some(result.ok().unwrap());
                }
                let player = players[player_id as usize].as_mut().unwrap();
                player.set_interact(true);
            }

            if mob_action_res.is_some() {
                string_result = format!("{}{}", string_result, mob_action_res.unwrap().string());
            }

            if sender.is_some() {
                sender.unwrap().send((string_result, Some(player_id))).unwrap();
            } else {
                let player = players[player_id as usize].as_ref().unwrap();
                player::send(player, string_result);
            }
        }
    });

    for stream in server.incoming() {
        match stream {
            Err(_) => println!("listen error"),
            Ok(stream) => {
                println!("connection from {} to {}",
                         stream.peer_addr().unwrap(),
                         stream.local_addr().unwrap());
                let send = send.clone();
                spawn(move|| {
                    handle_connection(stream, send);
                });
            }
        }
    }
}
