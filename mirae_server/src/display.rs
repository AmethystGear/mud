use crate::{
    entity::Entity, gamedata::gamedata::GameData, player::Player, rgb::RGB, vector3::Vector3,
    world::World,
};
use anyhow::{anyhow, Result};

#[derive(Debug)]
pub struct Bounds {
    posn: Vector3,
    width: usize,
    height: usize,
}

impl Bounds {
    fn bound(val: isize, min: isize, max: isize) -> isize {
        if val < min {
            min
        } else if val > max {
            max
        } else {
            val
        }
    }

    pub fn get_bounds_centered(posn: Vector3, size: usize, dim: Vector3) -> Self {
        let actual_size = 2 * (size as isize) + 1;
        let upper_left_x = (posn.x() - (size as isize))
            .max(0)
            .min(dim.x() - actual_size);
        let upper_left_y = (posn.y() - (size as isize))
            .max(0)
            .min(dim.y() - actual_size);
        let upper_left = Vector3::new(upper_left_x, upper_left_y, posn.z());

        Self {
            posn: upper_left,
            width: actual_size as usize,
            height: actual_size as usize,
        }
    }

    pub fn get_bounds(world: &World, posn: Vector3, width: usize, height: usize) -> Self {
        let bottom_right = posn + Vector3::new(width as isize, height as isize, 0);
        let bounded = Vector3::new(
            bottom_right.x().min(world.blocks().dim.x()),
            bottom_right.y().min(world.blocks().dim.y()),
            bottom_right.z(),
        );

        let diff = bounded - posn;

        let width = diff.x() as usize;
        let height = diff.y() as usize;
        Self {
            posn,
            width,
            height,
        }
    }

    pub fn in_bounds(&self, posn: Vector3) -> bool {
        let diff = posn - self.posn;
        diff.x() < self.width as isize
            && diff.x() >= 0
            && diff.y() < self.height as isize
            && diff.y() >= 0
            && diff.z() == 0
    }
}

pub struct Image {
    pub width: u8,
    pub height: u8,
    pub blocks: Vec<u8>,
    pub entities: Option<Vec<u8>>,
    pub players: Vec<u8>,
    pub res_is_1: bool,
}

impl Image {
    fn average_color(world: &World, bounds: &Bounds, gd: &GameData) -> Result<RGB> {
        let mut r = 0;
        let mut g = 0;
        let mut b = 0;
        for y in 0..bounds.height {
            for x in 0..bounds.width {
                let loc = bounds.posn + Vector3::new(x as isize, y as isize, 0);
                let rgb = world
                    .colors()
                    .get(loc)?
                    .mul(world.get_block_at(gd, loc)?.color);
                r += rgb.r as usize;
                g += rgb.g as usize;
                b += rgb.b as usize;
            }
        }
        r /= bounds.height * bounds.width;
        g /= bounds.height * bounds.width;
        b /= bounds.height * bounds.width;
        let rgb = RGB::new(r as u8, g as u8, b as u8);
        Ok(rgb)
    }

    fn display_blocks(
        world: &World,
        bounds: &Bounds,
        resolution: usize,
        g: &GameData,
    ) -> Result<Vec<u8>> {
        let mut display = Vec::new();
        if resolution != 1 {
            for j in 0..(bounds.height / resolution) {
                for i in 0..(bounds.width / resolution) {
                    let posn = bounds.posn
                        + Vector3::new((i * resolution) as isize, (j * resolution) as isize, 0);
                    let bounds = Bounds {
                        posn,
                        width: resolution,
                        height: resolution,
                    };
                    let rgb = Image::average_color(world, &bounds, g)?;
                    display.push(rgb.r);
                    display.push(rgb.g);
                    display.push(rgb.b);
                }
            }
        } else {
            for j in 0..bounds.height {
                for i in 0..bounds.width {
                    let loc = bounds.posn + Vector3::new(i as isize, j as isize, 0);
                    let mut rgb = world.colors().get(loc)?;
                    let block = world.get_block_at(g, loc)?;
                    if block.texture.is_none() {
                        rgb = rgb.mul(block.color);
                    }
                    display.push(rgb.r);
                    display.push(rgb.g);
                    display.push(rgb.b);
                }
            }

            for j in 0..bounds.height {
                for i in 0..bounds.width {
                    let loc = bounds.posn + Vector3::new(i as isize, j as isize, 0);
                    if let Some(texture_id) = g.block_id_to_img_id.get(&world.blocks().get(loc)?) {
                        display.push(*texture_id);
                    } else {
                        display.push(u8::MAX);
                    }
                }
            }
        }
        Ok(display)
    }

    fn display_entity(world: &World, bounds: &Bounds, g: &GameData) -> Result<Vec<u8>> {
        let mut display = Vec::new();
        for j in 0..bounds.height {
            for i in 0..bounds.width {
                let posn = bounds.posn + Vector3::new(i as isize, j as isize, 0);
                if let Some(curr) = world.mobs().get(posn)?.as_u16() {
                    let val = g
                        .mob_id_to_img_id
                        .get(&curr)
                        .expect("this should never happen");

                    display.push(val.clone());
                } else {
                    display.push(u8::MAX);
                }
            }
        }
        Ok(display)
    }

    fn display_players(
        players: &Vec<Option<Player>>,
        bounds: &Bounds,
        resolution: usize,
    ) -> Result<Vec<u8>> {
        let mut display = Vec::new();
        for j in 0..(bounds.height / resolution) {
            for i in 0..(bounds.width / resolution) {
                let posn = bounds.posn
                    + Vector3::new((i * resolution) as isize, (j * resolution) as isize, 0);
                let bounds = Bounds {
                    posn,
                    width: resolution,
                    height: resolution,
                };
                for id in 0..players.len() {
                    if let Some(p) = players[id].as_ref() {
                        if bounds.in_bounds(p.loc().clone()) {
                            display.push(id as u8);
                            display.push(i as u8);
                            display.push(j as u8);
                        }
                    }
                }
            }
        }
        Ok(display)
    }

    pub fn new(
        world: &World,
        players: &Vec<Option<Player>>,
        g: &GameData,
        bounds: &Bounds,
        resolution: usize,
    ) -> Result<Self> {
        let mut entities = None;
        if resolution == 0 {
            return Err(anyhow!("resolution cannot be 0"));
        } else if resolution == 1 {
            entities = Some(Image::display_entity(world, bounds, g)?);
        }
        let blocks = Image::display_blocks(world, bounds, resolution, g)?;
        let players = Image::display_players(players, bounds, resolution)?;
        Ok(Self {
            blocks,
            entities,
            players,
            width: (bounds.width / resolution) as u8,
            height: (bounds.height / resolution) as u8,
            res_is_1: resolution == 1,
        })
    }

    pub fn into_bytes(mut self) -> Vec<u8> {
        let mut vec = Vec::new();
        vec.push(self.width);
        vec.push(self.height);
        vec.push((self.players.len() / 3) as u8);
        if self.res_is_1 {
            vec.push(1);
        } else {
            vec.push(0);
        }
        vec.append(&mut self.players);
        vec.append(&mut self.blocks);
        if let Some(mut entities) = self.entities {
            vec.append(&mut entities);
        }
        vec
    }
}
