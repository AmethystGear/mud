use serde::Deserialize;
use std::ops::{Add, Mul, Sub};
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize)]
pub struct Vector3 {
    x: isize,
    y: isize,
    z: isize,
}

impl Vector3 {
    pub fn new(x: isize, y: isize, z: isize) -> Self {
        Vector3 { x, y, z }
    }

    pub fn x(&self) -> isize {
        self.x
    }

    pub fn y(&self) -> isize {
        self.y
    }

    pub fn z(&self) -> isize {
        self.z
    }

    pub fn dim(&self) -> isize {
        self.x * self.y * self.z
    }

    pub fn sqr_mag(&self) -> f64 {
        (self.x as f64).powi(2) + (self.y as f64).powi(2) + (self.z as f64).powi(2)
    }

    pub fn mag(&self) -> f64 {
        self.sqr_mag().sqrt()
    }

    pub fn set(&mut self, other : Vector3) {
        self.x = other.x();
        self.y = other.y();
        self.z = other.z();
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
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl Mul<f64> for Vector3 {
    type Output = Self;

    fn mul(self, other: f64) -> Self {
        Self::new(
            (self.x as f64 * other) as isize,
            (self.y as f64 * other) as isize,
            (self.z as f64 * other) as isize,
        )
    }
}

impl Mul<isize> for Vector3 {
    type Output = Self;

    fn mul(self, other: isize) -> Self {
        Self::new(self.x * other, self.y * other, self.z * other)
    }
}
