use macroquad::prelude::Color;

use crate::{materials::MaterialID, stains::Stain};

#[derive(Debug, Clone, Copy)]
pub struct Cell {
    pub material: MaterialID,
    pub stain: Option<Stain>,
    pub color: Color,
    pub awake: bool,
}
