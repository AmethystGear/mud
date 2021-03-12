use anyhow::Result;
use serde_jacl::de::from_str;
use std::fs;
use gamedata::{item::Item, gamedata::{DmgType, GameMode}, mobtemplate::MobTemplate, block::Block};
use world::World;

mod gamedata;
mod inventory;
mod noise;
mod rgb;
mod vector3;
mod world;
mod mob;

fn main() -> Result<()> {
    let m: GameMode = from_str(&fs::read_to_string("pvp/gamemode.jacl")?)?;
    let g = m.into_gamedata()?;

    let dmgs = g.dmg.clone();
    let dmg_types: Vec<&DmgType> = dmgs.iter().collect();
    let items: Vec<&Item> = g.items.values().collect();
    let mobs: Vec<&MobTemplate> = g.mob_templates.values().collect();
    let blocks: Vec<&Block> = g.blocks.values().collect();
    println!("{:?}", dmg_types);
    println!("");
    println!("{:?}", items);
    println!("");
    println!("{:?}", mobs);
    println!("");
    println!("{:?}", blocks);
    let _ = World::from_seed(0, &g)?;

    Ok(())
}
