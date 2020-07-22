extern crate ansi_term;
extern crate rstring_builder;

use crate::player;
use crate::player::Player;
use crate::world;
use crate::world::World;
use crate::stats;
use rstring_builder::StringBuilder;
use std::collections::HashMap;
use std::error::Error;


// returns the id of the block that is the most prevalent in the region.
fn get_majority_block_id(world : &World, x : u16, y : u16, square_length : u16) -> Result<u16, Box<dyn Error>> {
    let mut blocks = HashMap::new();
    for j in y..(y + square_length) {
        for i in x..(x + square_length) {
            let block = world::get_block(world, i, j);
            let id = stats::get(block, "id").ok_or_else(|| "block has no ID!")?;
            let id = id.as_int()?;
            blocks.insert(id, blocks.get(&id).unwrap_or(&0) + 1);
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
    return Ok(largest_id as u16);
}

pub fn display(world: &World, players : &Vec<Option<Player>>, 
               x_origin : u16,  y_origin : u16, x_length : u16, y_length : u16, resolution : u16) -> Result<StringBuilder, Box<dyn Error>> {
    println!("{}",x_origin);
    println!("{}",y_origin);
    let mut out = StringBuilder::new();
    let mut curr = StringBuilder::new();
    let mut prev_ascii : i64 = -1;
    let true_x_length = x_length / resolution;
    let true_y_length = y_length / resolution;
    for j in 0..true_y_length {
        out.append("|");
        for i in 0..true_x_length {
            let x = i * resolution + x_origin;
            let y = j * resolution + y_origin;
            let x_bound = (i + 1) * resolution + x_origin;
            let y_bound = (j + 1) * resolution + y_origin;

            let mut has_player = false;
            for k in 0..players.len() {
                match &players[k] {
                    Some (p) => {
                        let p_x = player::x(p)?;
                        let p_y = player::y(p)?;
                        if p_x >= x && p_x < x_bound && p_y >= y && p_y < y_bound {
                            let identity = stats::get(p.data(), "identity").ok_or_else(|| "player data doesn't have an identity!")?;
                            let identity = identity.as_box()?;
                            let id = stats::get(&identity, "id").ok_or_else(|| "player identity doesn't have an ID!")?;
                            let id = id.as_int()? as u8;
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
                    },
                    None => {}
                }
            }

            if !has_player {
                let block_id = get_majority_block_id(world, x, y, resolution)?;
                let block = world::get_block_by_id(world, block_id);
                let display = stats::get(block, "display").ok_or_else(|| "block doesn't have display color!")?;
                let display = display.as_int()?;
                if display != prev_ascii {
                    let style = ansi_term::Color::White.on(ansi_term::Color::Fixed(prev_ascii as u8));
                    let colored_text = style.paint(curr.string());
                    out.append(colored_text.to_string());
                    curr.clear();
                }
                if resolution == 1 {
                    let ent = world::get_entity_properties(world, x, y);
                    match ent {
                        Some(e) => {
                            let default = stats::Value::String("??".to_string());
                            let display = stats::get_or_else(e, "display", &default);
                            curr.append(display.as_string()?);
                        },
                        None => { 
                            curr.append("  ");
                        },
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
    return Ok(out);
}
