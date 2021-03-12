use crate::{gamedata, location::Vector2, world::{spawned::Located, world::World, player::Player}};
use anyhow::{anyhow, Error, Result};
use serde_json::json;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Eq, PartialEq)]
pub enum PacketType {
    Text = 0,
    Img = 1,
    Init = 2,
    Err = 3,
}

impl std::fmt::Display for PacketType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Packet {
    p_type: PacketType,
    content: Vec<u8>,
}

impl Packet {
    pub fn bytes(mut self) -> Vec<u8> {
        let mut header = format!("{}:{}:", self.p_type, self.content.len()).into_bytes();
        header.append(&mut (self.content));
        return header;
    }
}

pub struct PlayerOut {
    packets: VecDeque<Packet>,
}

impl PlayerOut {
    pub fn new() -> Self {
        PlayerOut {
            packets: VecDeque::new(),
        }
    }

    pub fn append<S: Into<String>>(&mut self, text: S) {
        if let Some(mut most_recent_pkt) = self.packets.pop_back() {
            if most_recent_pkt.p_type == PacketType::Text {
                most_recent_pkt
                    .content
                    .append(&mut text.into().into_bytes());
                self.packets.push_back(most_recent_pkt);
                return;
            }
        }
        self.packets.push_back(Packet {
            p_type: PacketType::Text,
            content: text.into().into_bytes(),
        });
    }

    pub fn append_err(&mut self, err: Error) {
        self.packets.push_back(Packet {
            p_type: PacketType::Err,
            content: err.to_string().into_bytes(),
        });
    }

    pub fn append_img(
        &mut self,
        world: &World,
        players: &Vec<Option<Player>>,
        img: Img,
    ) -> Result<()> {
        let mut vec_img: Vec<u8> = vec![];
        vec_img.push(0);
        vec_img.push(0);
        for k in 0..players.len() {
            match &players[k] {
                Some(p) => {
                    let loc = p.loc();
                    if loc.x >= img.origin.x
                        && loc.x < (img.origin.x + img.length.x)
                        && loc.y >= img.origin.y
                        && loc.y < (img.origin.y + img.length.y)
                    {
                        vec_img[1] += 1;
                        vec_img.push(0);
                        vec_img.push(p.id());
                        vec_img.push(0);
                        vec_img.push(((loc.x - img.origin.x) / img.resolution) as u8);
                        vec_img.push(0);
                        vec_img.push(((loc.y - img.origin.y) / img.resolution) as u8);
                    }
                }
                None => {}
            }
        }
        let bytes = (img.length.x / img.resolution).to_be_bytes();
        vec_img.push(bytes[0]);
        vec_img.push(bytes[1]);
        let display = display(world, img)?;
        let (mut block_vec, mut entity_vec) = (display.blocks, display.entities);
        let block_vec_len_bytes = ((block_vec.len() / 2) as u16).to_be_bytes();
        vec_img.push(block_vec_len_bytes[0]);
        vec_img.push(block_vec_len_bytes[1]);
        vec_img.append(&mut block_vec);
        let entity_vec_len_bytes = ((entity_vec.len() / 2) as u16).to_be_bytes();
        vec_img.push(entity_vec_len_bytes[0]);
        vec_img.push(entity_vec_len_bytes[1]);
        vec_img.append(&mut entity_vec);

        let pkt = Packet {
            p_type: PacketType::Img,
            content: vec_img,
        };
        self.packets.push_back(pkt);
        return Ok(());
    }

    pub fn get_pkt(&mut self) -> Option<Packet> {
        return self.packets.pop_front();
    }

    pub fn append_player_out(&mut self, mut p_out: PlayerOut) {
        let mut pkt = p_out.get_pkt();
        while let Some(p) = pkt {
            self.packets.push_back(p);
            pkt = p_out.get_pkt();
        }
    }

    pub fn add_pkt(&mut self, p: Packet) {
        self.packets.push_back(p);
    }
}

pub fn get_init(world: &World) -> Result<Packet> {
    let mut data = json!({
        "block_display" : {},
        "entity_display" : {}
    });
    let displays = [
        ("block_display", &world.blocks),
        ("entity_display", &world.entities),
    ];
    for d in displays.iter() {
        let (entry, map) = d;
        let (entry, map) = (*entry, *map);
        for i in 0..map.max() {
            let name = map
                .id_to_name()
                .get(&i)
                .ok_or_else(|| anyhow!("no mapping for id {} in map {:?}", i, map))?;
            let display = match entry {
                "block_display" => {
                    let block = gamedata::GAMEDATA
                        .blocks
                        .get(name)
                        .ok_or_else(|| anyhow!("no block with name {}", name))?;
                    Ok(&block.display)
                }
                "entity_display" => {
                    let entity = gamedata::GAMEDATA
                        .entities
                        .get(name)
                        .ok_or_else(|| anyhow!("no entity with name {}", name))?;
                    Ok(&entity.display)
                }
                _ => Err(anyhow!("likely, a name was mispelled")),
            }?;
            data[entry]
                .as_object_mut()
                .ok_or_else(|| anyhow!("badly formatted json value"))?
                .insert(format!("{}", i), json!(display));
        }
    }
    return Ok(Packet {
        p_type: PacketType::Init,
        content: data.to_string().into_bytes(),
    });
}

pub struct Img {
    pub origin: Vector2,
    pub length: Vector2,
    pub resolution: u16,
}

pub struct Display {
    blocks: Vec<u8>,
    entities: Vec<u8>,
}

/// returns the id of the most common block in the region.
fn get_majority_block_id(world: &World, x: u16, y: u16, square_length: u16) -> Result<u16> {
    let mut blocks = HashMap::new();
    for j in y..(y + square_length) {
        for i in x..(x + square_length) {
            let id = world
                .blocks
                .get_id(Vector2::new(i, j))?
                .ok_or_else(|| anyhow!("no block at location!"))?;
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
    return Ok(largest_id);
}

/// returns a display of a portion of the world map.
pub fn display(world: &World, img: Img) -> Result<Display> {
    let x_origin = img.origin.x;
    let y_origin = img.origin.y;
    let x_length = img.length.x;
    let y_length = img.length.y;
    let resolution = img.resolution;
    let mut blocks = vec![];
    let mut entities = vec![];
    let true_x_length = x_length / resolution;
    let true_y_length = y_length / resolution;
    for j in 0..true_y_length {
        for i in 0..true_x_length {
            let x = i * resolution + x_origin;
            let y = j * resolution + y_origin;
            let block_id = get_majority_block_id(world, x, y, resolution)?;
            let bytes = block_id.to_be_bytes();
            blocks.push(bytes[0]);
            blocks.push(bytes[1]);
            if resolution == 1 {
                let ent = world.entities.get_id(Vector2::new(x, y))?;
                match ent {
                    Some(ent) => {
                        let bytes = ent.to_be_bytes();
                        entities.push(bytes[0]);
                        entities.push(bytes[1]);
                    }
                    None => {
                        entities.push(u8::MAX);
                        entities.push(u8::MAX);
                    }
                }
            }
        }
    }
    return Ok(Display { blocks, entities });
}
