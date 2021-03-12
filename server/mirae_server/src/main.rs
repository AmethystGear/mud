use std::env;

mod deser;
mod gamedata;
mod inventory;
mod location;
mod playerout;
mod requests;
mod trades;

pub fn main() {
    let args: Vec<String> = env::args().collect();
}
