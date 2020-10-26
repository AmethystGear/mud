use std::env;
use std::fs;

mod deser;
mod gamedata;
mod inventory;
mod location;
mod requests;
mod trades;
mod world;
mod playerout;

pub fn main() {
    let args: Vec<String> = env::args().collect();
}
