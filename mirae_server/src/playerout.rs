use crate::display::display;
use crate::display::Img;
use crate::player;
use crate::player::Player;
use crate::stats;
use crate::world;
use crate::world::World;
use serde_json::json;
use std::collections::VecDeque;
use std::error::Error;

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

    pub fn append_err(&mut self, err: Box<dyn Error>) {
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
    ) -> Result<(), Box<dyn Error>> {
        let mut vec_img: Vec<u8> = vec![];
        vec_img.push(0);
        vec_img.push(0);
        for k in 0..players.len() {
            match &players[k] {
                Some(p) => {
                    let p_x = player::x(p)?;
                    let p_y = player::y(p)?;
                    if p_x >= img.x_origin
                        && p_x < (img.x_origin + img.x_length)
                        && p_y >= img.y_origin
                        && p_y < (img.y_origin + img.y_length)
                    {
                        let identity = stats::get(p.data(), "identity")?;
                        let identity = identity.as_box()?;
                        let id = stats::get(&identity, "id")?;
                        let id = id.as_int()? as u8;
                        vec_img[1] += 1;
                        vec_img.push(0);
                        vec_img.push(id);
                        vec_img.push(0);
                        vec_img.push(((p_x - img.x_origin) / img.resolution) as u8);
                        vec_img.push(0);
                        vec_img.push(((p_y - img.y_origin) / img.resolution) as u8);
                    }
                }
                None => {}
            }
        }
        let bytes = (img.x_length / img.resolution).to_be_bytes();
        vec_img.push(bytes[0]);
        vec_img.push(bytes[1]);
        let (mut block_vec, mut entity_vec) = display(world, img)?;
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

    /*
    pub fn append_player_out(&mut self, mut p_out: PlayerOut) {
        let most_recent_pkt = self.packets.pop_back();
        let first_p_out = p_out.get_pkt();
        if most_recent_pkt.is_none() {
            self.packets = p_out.packets;
            return;
        }
        if first_p_out.is_none() {
            return;
        }
        let mut first_p_out = first_p_out.expect("checked for none case above");
        let mut most_recent_pkt = most_recent_pkt.expect("check for none case anove");

        if most_recent_pkt.p_type == PacketType::Text && first_p_out.p_type == PacketType::Text {
            most_recent_pkt.content.append(&mut first_p_out.content);
        }
        self.packets.push_back(most_recent_pkt);
        let mut pkt = Some(first_p_out);
        while pkt.is_some() {
            self.packets
                .push_back(pkt.expect("pkt should never be none in loop"));
            pkt = p_out.get_pkt();
        }
    }
    */

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

pub fn get_init(world: &World) -> Result<Packet, Box<dyn Error>> {
    let mut init = stats::Stats::new();
    stats::set(
        &mut init,
        "default_mob",
        stats::Value::String("??".to_string()),
    );

    let mut data = json!({
        "default_entity" : "??",
        "block_display" : {},
        "entity_display" : {}
    });
    for i in 0..(world.max_block_id()) {
        let block = world::get_block_by_id(world, i)?;
        let display = stats::get(block, "display")?.as_int()? as u16;
        data["block_display"]
            .as_object_mut()
            .ok_or("badly formatted json value")?
            .insert(format!("{}", i), json!(display));
    }
    for i in 0..(world.max_entity_id()) {
        let entity = world::get_entity_properties_by_id(world, i)?;
        if stats::has_var(entity, "display") {
            let display = stats::get(entity, "display")?.as_string()?;
            data["entity_display"]
                .as_object_mut()
                .ok_or("badly formatted json value")?
                .insert(format!("{}", i), json!(display));
        }
    }
    return Ok(Packet {
        p_type: PacketType::Init,
        content: data.to_string().into_bytes(),
    });
}
