use crate::deser::gamemode::{GameData, GameMode};
use lazy_static::lazy_static;
use serde_jacl::de::from_str;
use std::fs;

lazy_static! {
    pub static ref GAMEDATA: GameData = {
        let m: GameMode = from_str(&fs::read_to_string("gamemode.jacl").unwrap()).unwrap();
        (&m).into().unwrap()
    };
}
