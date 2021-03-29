use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGB {
    pub fn new(r: u8, g: u8, b: u8) -> RGB {
        RGB { r, g, b }
    }

    pub fn black() -> RGB {
        RGB::new(0, 0, 0)
    }

    pub fn white() -> RGB {
        RGB::new(255, 255, 255)
    }

    pub fn mul(&self, other: RGB) -> RGB {
        let r = self.r as u16;
        let g = self.g as u16;
        let b = self.b as u16;
        let r_o = other.r as u16;
        let g_o = other.g as u16;
        let b_o = other.b as u16;
        RGB::new(
            (r * r_o / 255) as u8,
            (g * g_o / 255) as u8,
            (b * b_o / 255) as u8,
        )
    }

    pub fn scale(&self, other: f64) -> RGB {
        RGB::new(
            ((self.r as f64) * other).min(255.0) as u8,
            ((self.g as f64) * other).min(255.0) as u8,
            ((self.b as f64) * other).min(255.0) as u8,
        )
    }

    pub fn add(&self, other: RGB) -> RGB {
        RGB::new(
            (self.r as u16 + other.r as u16).min(255) as u8,
            (self.g as u16 + other.g as u16).min(255) as u8,
            (self.b as u16 + other.b as u16).min(255) as u8,
        )
    }
}
