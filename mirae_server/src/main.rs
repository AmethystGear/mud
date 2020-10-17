use std::env;
use std::fs;

mod deser;
mod gamedata;
mod inventory;
mod location;
mod requests;
mod trades;
mod world;

pub fn main() {
    let args: Vec<String> = env::args().collect();
    let gamemodes =
        deser::gamemode::get_gamemodes(&fs::read_to_string("gamemodes.jacl").unwrap()).unwrap();
    let gamemode = gamemodes.get(&args[0]).unwrap();
    let gamedata = gamemode.into().unwrap();
    gamedata::assign(gamedata).unwrap();
}
