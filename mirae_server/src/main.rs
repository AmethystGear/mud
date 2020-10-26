use std::env;
use std::fs;

mod deser;
mod gamedata;
mod inventory;
mod location;
mod playerout;
mod requests;
mod trades;
mod world;

pub fn main() {
    let args: Vec<String> = env::args().collect();
}
