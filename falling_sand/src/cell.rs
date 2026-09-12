#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        return Self { r, g, b, a };
    }
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        return Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        };
    }
}
impl std::ops::Mul<f32> for Color {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Color {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
            a: self.a,
        }
    }
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(remote = "Color")]
struct ColorDef {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

use crate::{
    materials::{Behavior, MaterialID},
    stains::Stain,
};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Cell {
    pub material: MaterialID,
    pub stain: Option<Stain>,
    #[serde(with = "ColorDef")]
    pub color: Color,
    pub awake: bool,
}
impl Cell {
    pub fn behaviors(&self) -> Behavior {
        return self.material.properties().behavior
            | self
                .stain
                .map(|s| s.kind.behavior())
                .unwrap_or(Behavior::empty());
    }
}
