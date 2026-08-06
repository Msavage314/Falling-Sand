use macroquad::prelude::Color;

use crate::{
    materials::{Behavior, MaterialID},
    stains::Stain,
};

#[derive(Debug, Clone, Copy)]
pub struct Cell {
    pub material: MaterialID,
    pub stain: Option<Stain>,
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
