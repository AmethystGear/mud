use anyhow::Result;
use gamedata::gamedata::GameMode;
use serde_jacl::de::from_str;
use std::{time::Instant, fs};
use world::World;

mod display;
mod gamedata;
mod inventory;
mod mob;
mod noise;
mod playerout;
mod rgb;
mod stat;
mod vector3;
mod world;

fn main() -> Result<()> {
    let m: GameMode = from_str(&fs::read_to_string("pvp/gamemode.jacl")?)?;
    let g = m.into_gamedata()?;
    println!("read game data");
    println!("begin world generation...");
    let start = Instant::now();
    let world = World::from_seed(0, &g)?;
    let duration = start.elapsed();
    println!("generated world in {:?}", duration);
    loop {
        
    }
}
