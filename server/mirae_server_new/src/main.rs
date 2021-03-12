use anyhow::Result;
use block::Block;
use gamedata::{DmgType, GameMode};
use item::Item;
use mob::MobTemplate;
use serde_jacl::de::from_str;
use std::fs;

mod block;
mod gamedata;
mod inventory;
mod item;
mod mob;
mod noise;
mod rgb;
mod serde_defaults;
mod terrain;
mod vector3;
mod world;

fn main() -> Result<()> {
    let m: GameMode = from_str(&fs::read_to_string("pvp/gamemode.jacl")?)?;
    let gamedata = m.into_gamedata()?;

    let dmgs = gamedata.dmg.clone();
    let dmg_types: Vec<&DmgType> = dmgs.iter().collect();
    let items: Vec<&Item> = gamedata.items.values().collect();
    let mobs: Vec<&MobTemplate> = gamedata.mob_templates.values().collect();
    let blocks: Vec<&Block> = gamedata.blocks.values().collect();
    println!("{:?}", dmg_types);
    println!("");
    println!("{:?}", items);
    println!("");
    println!("{:?}", mobs);
    println!("");
    println!("{:?}", blocks);
    Ok(())
}
