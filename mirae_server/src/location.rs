#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Copy, Clone)]
pub struct Vector2 {
    pub x: u16,
    pub y: u16,
}

impl Vector2 {
    pub fn new(x: u16, y: u16) -> Self {
        Vector2 { x, y }
    }

    pub fn sqr_dist(&self, other: &Vector2) -> f64 {
        let x_delta = self.x as f64 - other.x as f64;
        let y_delta = self.y as f64 - other.y as f64;
        return x_delta * x_delta + y_delta * y_delta;
    }

    pub fn dist(&self, other: &Vector2) -> f64 {
        return self.sqr_dist(other).sqrt();
    }

    pub fn sqr_mag(&self) -> f64 {
        return self.sqr_dist(&Vector2::new(0, 0));
    }

    pub fn mag(&self) -> f64 {
        return self.dist(&Vector2::new(0, 0));
    }

    pub fn manhattan_dist(&self, other: &Vector2) -> u16 {
        return self.x - other.x + self.y - other.y;
    }

    pub fn manhattan_mag(&self) -> u16 {
        return self.manhattan_dist(&Vector2::new(0, 0));
    }
}
