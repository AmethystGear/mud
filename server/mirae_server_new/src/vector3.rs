use serde::Deserialize;
use std::ops::{Add, Mul, Sub};
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize)]
pub struct Vector3 {
    x: usize,
    y: usize,
    z: usize,
}

impl Vector3 {
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        Vector3 { x, y, z }
    }

    pub fn x(&self) -> usize {
        self.x
    }

    pub fn y(&self) -> usize {
        self.y
    }

    pub fn z(&self) -> usize {
        self.z
    }

    pub fn dim(&self) -> usize {
        self.x * self.y * self.z
    }

    pub fn sqr_mag(&self) -> f64 {
        (self.x as f64).powi(2) + (self.y as f64).powi(2) + (self.z as f64).powi(2)
    }

    pub fn mag(&self) -> f64 {
        self.sqr_mag().sqrt()
    }
}

impl Add for Vector3 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl Sub for Vector3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(
            (self.x as isize - other.x as isize).max(0) as usize,
            (self.y as isize - other.y as isize).max(0) as usize,
            (self.z as isize - other.z as isize).max(0) as usize,
        )
    }
}

impl Mul<f64> for Vector3 {
    type Output = Self;

    fn mul(self, other: f64) -> Self {
        Self::new(
            (self.x as f64 * other) as usize,
            (self.y as f64 * other) as usize,
            (self.z as f64 * other) as usize,
        )
    }
}

impl Mul<usize> for Vector3 {
    type Output = Self;

    fn mul(self, other: usize) -> Self {
        Self::new(self.x * other, self.y * other, self.z * other)
    }
}
