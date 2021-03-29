#![allow(dead_code)]
use anyhow::{anyhow, Result};
use display::{Bounds, Image};
use flo_draw::{
    canvas::{Color, GraphicsContext},
    create_canvas_window, with_2d_graphics,
};
use gamedata::gamedata::GameMode;
use io::BufRead;
use serde_jacl::{de::from_str, structs::Literal};
use std::{fs, io, time::Instant};
use vector3::Vector3;
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

const DEBUG_BLOCK_SIZE: f32 = 10.0;
const CANVAS_SIZE: f32 = 1000.0;

fn draw(image: Image) {
    with_2d_graphics(move || {
        let canvas = create_canvas_window("debug");
        canvas.draw(|gc| {
            gc.clear_canvas(Color::Rgba(1.0, 1.0, 1.0, 1.0));
            gc.canvas_height(CANVAS_SIZE);
            gc.center_region(0.0, 0.0, CANVAS_SIZE, CANVAS_SIZE);

            for y in 0..image.height {
                for x in 0..image.width {
                    gc.new_path();
                    let x_c = (x as f32) * DEBUG_BLOCK_SIZE;
                    let y_c = (y as f32) * DEBUG_BLOCK_SIZE;
                    gc.move_to(x_c, y_c);
                    gc.line_to(x_c + DEBUG_BLOCK_SIZE, y_c);
                    gc.line_to(x_c + DEBUG_BLOCK_SIZE, y_c + DEBUG_BLOCK_SIZE);
                    gc.line_to(x_c, y_c + DEBUG_BLOCK_SIZE);
                    gc.line_to(x_c, y_c);
                    let index = ((x as usize) + (y as usize) * (image.width as usize)) * 3;
                    let r = (image.rgb[index] as f32) / 255.0;
                    let g = (image.rgb[index + 1] as f32) / 255.0;
                    let b = (image.rgb[index + 2] as f32) / 255.0;
                    gc.fill_color(Color::Rgba(r, g, b, 1.0));
                    gc.fill();
                }
            }
        });
    });
}

fn main() -> Result<()> {
    let m: GameMode = from_str(&fs::read_to_string("pvp/gamemode.jacl")?)?;
    let g = m.into_gamedata()?;
    println!("read game data");
    println!("begin world generation...");
    let start = Instant::now();
    let world = World::from_seed(0, &g)?;
    let duration = start.elapsed();
    println!("generated world in {:?}", duration);

    println!("{:?}", world.blocks().dim);

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let params: Vec<Literal> = from_str(&line?)?;
        if params.len() > 0 {
            match &params[0] {
                Literal::String(s) => {
                    match s.as_str() {
                        "map" => {
                            let size = &params[1];
                            let size = size.as_int().unwrap_or(&10);

                            let level = params[2].as_int().unwrap_or(&0);
                            let resolution = world.blocks().dim.x() / (size.clone() as usize);
                            let bounds = Bounds::get_bounds(
                                &world,
                                Vector3::new(0, 0, *level as usize),
                                world.blocks().dim.x(),
                                world.blocks().dim.y(),
                            );
                            let image = Image::new(&world, &g, &bounds, resolution)?;
                            draw(image);
                            Ok(())
                        }

                        "look" => {
                            let x = params[1].as_int().unwrap_or(&0);
                            let y = params[2].as_int().unwrap_or(&0);
                            let z = params[3].as_int().unwrap_or(&0);
                            let width = params[4].as_int().unwrap_or(&0);
                            let height = params[5].as_int().unwrap_or(&0);
                            let posn = Vector3::new(*x as usize, *y as usize, *z as usize);
                            let bounds =
                                Bounds::get_bounds(&world, posn, *width as usize, *height as usize);
                            
                            println!("bounds: {:?}", bounds);
                            let image = Image::new(&world, &g, &bounds, 1)?;
                            draw(image);                            
                            Ok(())
                        }
                        _ => Err(anyhow!("invalid command!")),
                    }?;
                    Ok(())
                }
                _ => Err(anyhow!("command must start with a string")),
            }?;
        }
    }
    Ok(())
}
