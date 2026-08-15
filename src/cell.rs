use macroquad::prelude::Color;
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
