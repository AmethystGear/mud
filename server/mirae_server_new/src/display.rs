use crate::{gamedata::gamedata::GameData, rgb::RGB, vector3::Vector3, world::World};
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
        // calculate upper x and y bounds
        let actual_size = 2 * (size as isize) + 1;
        let upper_bound = |dim| (dim as isize) - (actual_size as isize);
        let upper_x_bound = upper_bound(dim.x());
        let upper_y_bound = upper_bound(dim.y());

        let bound_upper_left =
            |posn, upper_bound| Bounds::bound((posn as isize) - (size as isize), 0, upper_bound);

        let upper_left_x = bound_upper_left(posn.x(), upper_x_bound);
        let upper_left_y = bound_upper_left(posn.y(), upper_y_bound);

        let upper_left = Vector3::new(upper_left_x, upper_left_y, posn.z());

        Self {
            posn: upper_left,
            width: actual_size.min(dim.x()) as usize,
            height: actual_size.min(dim.z()) as usize,
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
            height
        }
    }
}

pub struct Image {
    pub width : u8,
    pub height : u8,
    pub rgb : Vec<u8>,
    pub entities : Option<Vec<u8>>
}

impl Image {
    fn average_color(world: &World, bounds: &Bounds) -> Result<RGB> {
        let mut r = 0;
        let mut g = 0;
        let mut b = 0;
        for y in 0..bounds.height {
            for x in 0..bounds.width {
                let loc = bounds.posn + Vector3::new(x as isize, y as isize, 0);
                let rgb = world.colors().get(loc)?;
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

    fn display_rgb(world: &World, bounds: &Bounds, resolution: usize) -> Result<Vec<u8>> {
        let mut display = Vec::new();
        for j in 0..(bounds.height / resolution) {
            for i in 0..(bounds.width / resolution) {
                let posn = bounds.posn + Vector3::new((i * resolution) as isize, (j * resolution) as isize, 0);
                let bounds = Bounds {
                    posn,
                    width: resolution,
                    height: resolution,
                };
                let rgb = Image::average_color(world, &bounds)?;
                display.push(rgb.r);
                display.push(rgb.g);
                display.push(rgb.b);
            }
        }
        Ok(display)
    }

    fn display_entity(world: &World, bounds: &Bounds, g: &GameData) -> Result<Vec<u8>> {
        let mut display = Vec::new();
        for j in 0..bounds.height {
            for i in 0..bounds.width {
                let posn = bounds.posn + Vector3::new( i as isize, j as isize, 0);
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

    pub fn new(world : &World, g : &GameData, bounds: &Bounds, resolution: usize) -> Result<Self> {
        let mut entities = None;
        if resolution == 0 {
            return Err(anyhow!("resolution cannot be 0"));
        } else if resolution == 1 {
            entities = Some(Image::display_entity(world, bounds, g)?);
        }
        let rgb = Image::display_rgb(world, bounds, resolution)?;
        Ok(Self {
            rgb,
            entities,
            width: (bounds.width / resolution) as u8,
            height: (bounds.height / resolution) as u8
        })
    }

    pub fn into_bytes(mut self) -> Vec<u8> {
        let mut vec = Vec::new();
        vec.push(self.width);
        vec.append(&mut self.rgb);
        if let Some(mut entities) = self.entities {
            vec.append(&mut entities);
        }
        vec
    }
}
