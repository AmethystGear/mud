extern crate ansi_term;
extern crate rstring_builder;

use crate::scanner::Param;
use crate::action::Action;
use crate::player::Player;
use crate::entities::SpawnedEntities;
use crate::playerout::PlayerOut;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::thread::spawn;
use std::io::{BufReader, BufRead, BufWriter};
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::u8;

use std::error::Error;

mod scanner;
mod stats;
mod action;
mod world;
mod perlin_noise;
mod player;
mod display;
mod entities;
mod playerout;

type ConnOut = (Option<String>, Option<Vec<Param>>, Option<Action>, Option<Sender<(PlayerOut, Option<u8>)>>, Option<u8>);
type ConnIn = (PlayerOut, Option<u8>);

fn handle_connection(stream: TcpStream, channel : Sender<ConnOut>) {
    let action_map = action::get_action_map();
    let (send, recv) : (Sender<ConnIn>, Receiver<ConnIn>) = mpsc::channel();
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

    // packet send step
    spawn(move || {
        loop {
            let (mut response, _) = recv.recv().unwrap();
            let mut pkt = response.get_pkt();
            while pkt.is_some() {
                writer.write_all(&pkt.unwrap().bytes()).unwrap();
                pkt = response.get_pkt();
            }
            writer.flush().unwrap();
        }
    });


    let mut last_res : Option<(String, Vec<Param>, Action)> = None;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let action_res : Result<(String, Vec<Param>, Action), Box<dyn Error>>;
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
                println!("\n\nERROR: {}\n\n", res.err().unwrap());
            }
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
    let (send, recv) : (Sender<ConnOut>, Receiver<ConnOut>) = mpsc::channel();

    spawn(move || {
        let mut world : world::World = world::from_seed(0).ok().unwrap();
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
                    let mut p_out = PlayerOut::new();
                    p_out.append_err("Cannot enter game! There are already 256 players on the server!".to_string());
                    sender.send((p_out, None)).unwrap();
                    continue;
                }
                let player = player::from(0, 0, id.unwrap(), sender.clone());
                if player.is_err() {
                    println!("failed to create player!");
                    continue;
                }
                let mut player = player.expect("just checked that player is not err, so this should never fail");
                let res = player::respawn(&mut player, &world);
                if res.is_err() {
                    println!("failed to respawn player!");
                    continue;
                }

                println!("{}", id.unwrap());
                player_id = id.unwrap();
                players[player_id as usize] = Some(player);
                let mut p_out = PlayerOut::new();
                p_out.add_pkt(playerout::get_init(&world).unwrap());
                sender.send((p_out, Some(player_id))).unwrap();
                continue;
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
                let result = action.run(Some(&mut spawned_entities), Some(&action_map), Some(keyword), Some(&params),
                                        Some(player_id), Some(&mut players), Some(&mut world));
                let player = players[player_id as usize].as_ref().unwrap();
                let x_ = player::x(&player);
                let y_ = player::y(&player);
                if x_.is_err() || y_.is_err() {
                    println!("{:?}", x_);
                    println!("{:?}", y_);
                    continue;
                }
                x = x_.unwrap();
                y = y_.unwrap();
                if !entities::has_entity(&spawned_entities, x, y) && world::has_entity(&world, x, y) {
                    let name = world::get_entity_name(&world, x, y).unwrap();
                    let stats = world::get_entity_properties(&world, x, y).unwrap().clone();
                    entities::spawn(stats, x, y, name, &mut spawned_entities, &mut world).unwrap();
                }
                match result {
                    Ok(ok) => {
                        res = ok;
                    }
                    Err(err) => {
                        res = PlayerOut::new();
                        res.append(err.to_string());
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
                let entity_action : Action = entities::get_entity_action(&mut spawned_entities, "interact".to_string(), x, y).unwrap();
                let result = entity_action.run(Some(&mut spawned_entities), None, None, None, Some(player_id), Some(&mut players), Some(&mut world));
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
                        println!("could not send to player!");
                    }
                }
                None => {println!("Invalid player id {}", player_id)}
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
