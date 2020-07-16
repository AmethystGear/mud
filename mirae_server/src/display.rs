extern crate ansi_term;
extern crate rstring_builder;

use crate::player;
use crate::player::Player;
use crate::world;
use crate::world::World;
use crate::stats;
use rstring_builder::StringBuilder;
use std::collections::HashMap;

// returns the id of the block that is the most prevalent in the region.
fn get_majority_block_id(world : &World, x : u16, y : u16, square_length : u16) -> u16 {
    let mut blocks = HashMap::new();
    for j in y..(y + square_length) {
        for i in x..(x + square_length) {
            let block = world::get_block(world, i, j);
            let id = stats::get(block, "id").unwrap().as_int();
            if !blocks.contains_key(&id) {
                blocks.insert(id, 1);
            } else {
                blocks.insert(id, blocks.get(&id).unwrap() + 1);
            }
        }
    }
    let mut largest_id = -1;
    let mut largest_num = 0;
    for (id, num) in blocks {
        if num > largest_num {
            largest_num = num;
            largest_id = id;
        }
    }
    return largest_id as u16;
}

pub fn display(world: &World, players : &Vec<Option<Player>>, x_origin : u16,  y_origin : u16, square_length : u16, resolution : u16) -> StringBuilder {
    println!("{}",x_origin);
    println!("{}",y_origin);
    let mut out = StringBuilder::new();
    let mut curr = StringBuilder::new();
    let mut prev_ascii = -1;
    let true_length = square_length / resolution;
    for j in 0..true_length {
        out.append("|");
        for i in 0..true_length {
            let x = i * resolution + x_origin;
            let y = j * resolution + y_origin;
            let x_bound = (i + 1) * resolution + x_origin;
            let y_bound = (j + 1) * resolution + y_origin;

            let mut has_player = false;
            for k in 0..players.len() {
                if players[k].is_some() {
                    let p = players[k].as_ref().unwrap();
                    let p_x = player::x(p);
                    let p_y = player::y(p);
                    if p_x >= x && p_x < x_bound && p_y >= y && p_y < y_bound {
                        let identity = stats::get(p.data(), "identity").unwrap().as_box();
                        let id = stats::get(&identity, "id").unwrap().as_int() as u8;
                        let mut id_str = format!("{:x}", id);
                        if id_str.len() == 1 {
                            id_str = format!("0{}", id_str);
                        }
                        let style = ansi_term::Color::White.on(ansi_term::Color::Fixed(prev_ascii as u8));
                        let colored_text = style.paint(curr.string());
                        out.append(colored_text.to_string());
                        out.append(id_str);
                        curr.clear();
                        has_player = true;
                        prev_ascii = -1;
                        break;
                    }
                }
            }

            if !has_player {
                let block_id = get_majority_block_id(world, x, y, resolution);
                let block = world::get_block_by_id(world, block_id);
                let display = stats::get(block, "display").unwrap().as_int();
                if display != prev_ascii {
                    let style = ansi_term::Color::White.on(ansi_term::Color::Fixed(prev_ascii as u8));
                    let colored_text = style.paint(curr.string());
                    out.append(colored_text.to_string());
                    curr.clear();
                }
                if resolution == 1 {
                    let e = world::get_entity_properties(world, x, y);
                    if e.is_none() {
                        curr.append("  ");
                    } else {
                        let e = e.unwrap();
                        if stats::has_var(e, "display") {
                            curr.append(stats::get(e, "display").unwrap().as_string());
                        } else {
                            curr.append("??");
                        }
                    }
                } else {
                    curr.append("  ");
                }
                prev_ascii = display;
            }
        }
        let style = ansi_term::Color::White.on(ansi_term::Colour::Fixed(prev_ascii as u8));
        let colored_text = style.paint(curr.string());
        out.append(colored_text.to_string());
        out.append("|\n");
        curr.clear();
    }
    return out;
}
