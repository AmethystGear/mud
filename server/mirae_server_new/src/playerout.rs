use crate::{gamedata::gamedata::GameData, world::MobU16, display::Image};
use anyhow::{anyhow, Result};
use serde_json::json;
use std::{
    collections::{HashMap, VecDeque},
    error::Error,
};

#[derive(Debug, Eq, PartialEq, Clone)]
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

#[derive(Debug, Clone)]
pub struct Packet {
    pub p_type: PacketType,
    pub content: Vec<u8>,
}

impl Packet {
    pub fn bytes(mut self) -> Vec<u8> {
        let mut header = format!("{}:{}:", self.p_type, self.content.len()).into_bytes();
        header.append(&mut (self.content));
        header
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

    pub fn append_text<S: Into<String>>(&mut self, text: S) {
        if let Some(mut most_recent_pkt) = self.packets.pop_back() {
            if most_recent_pkt.p_type == PacketType::Text {
                most_recent_pkt
                    .content
                    .append(&mut text.into().into_bytes());
                self.packets.push_back(most_recent_pkt);
                return;
            }
        }
        self.add_pkt(Packet {
            p_type: PacketType::Text,
            content: text.into().into_bytes(),
        });
    }

    pub fn append_err(&mut self, err: Box<dyn Error>) {
        self.add_pkt(Packet {
            p_type: PacketType::Err,
            content: err.to_string().into_bytes(),
        });
    }

    pub fn append_img(&mut self, img : Image) {
        self.add_pkt(Packet {
            p_type : PacketType::Img,
            content: img.into_bytes()
        });
    }

    pub fn get_pkt(&mut self) -> Option<Packet> {
        self.packets.pop_front()
    }

    pub fn append_player_out(&mut self, mut p_out: PlayerOut) {
        while let Some(p) = p_out.get_pkt() {
            self.add_pkt(p);
        }
    }

    pub fn add_pkt(&mut self, p: Packet) {
        self.packets.push_back(p);
    }
}
