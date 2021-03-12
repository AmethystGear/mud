use crate::world;
use crate::world::World;
use std::collections::HashMap;
use std::error::Error;

pub struct Img {
    pub x_origin: u16,
    pub y_origin: u16,
    pub x_length: u16,
    pub y_length: u16,
    pub resolution: u16,
}

/// returns the id of the most common block in the region.
fn get_majority_block_id(world: &World, x: u16, y: u16, square_length: u16) -> u16 {
    let mut blocks = HashMap::new();
    for j in y..(y + square_length) {
        for i in x..(x + square_length) {
            let id = world::get_block_id(world, i, j);
            blocks.insert(id, blocks.get(&id).unwrap_or(&0) + 1);
        }
    }
    let mut largest_id = 0;
    let mut largest_num = 0;
    for (id, num) in blocks {
        if num > largest_num {
            largest_num = num;
            largest_id = id;
        }
    }
    return largest_id;
}

/// returns a display of a portion of the world map.
pub fn display(world: &World, img: Img) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
    let x_origin = img.x_origin;
    let y_origin = img.y_origin;
    let x_length = img.x_length;
    let y_length = img.y_length;
    let resolution = img.resolution;
    let mut block_vec = vec![];
    let mut entity_vec = vec![];
    let true_x_length = x_length / resolution;
    let true_y_length = y_length / resolution;
    for j in 0..true_y_length {
        for i in 0..true_x_length {
            let x = i * resolution + x_origin;
            let y = j * resolution + y_origin;
            let block_id = get_majority_block_id(world, x, y, resolution);
            let bytes = block_id.to_be_bytes();
            block_vec.push(bytes[0]);
            block_vec.push(bytes[1]);
            if resolution == 1 {
                let ent = world::get_entity_id(world, x, y);
                match ent {
                    u16::MAX => {
                        entity_vec.push(u8::MAX);
                        entity_vec.push(u8::MAX);
                    }
                    _ => {
                        let bytes = ent.to_be_bytes();
                        entity_vec.push(bytes[0]);
                        entity_vec.push(bytes[1]);
                    }
                }
            }
        }
    }
    return Ok((block_vec, entity_vec));
}
