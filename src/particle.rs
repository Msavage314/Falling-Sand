use egui_macroquad::egui::Vec2;
use serde::{Deserialize, Serialize};

use crate::materials::MaterialID;

#[derive(Serialize, Deserialize)]
#[serde(remote = "Vec2")]
struct Vec2Def {
    pub x: f32,

    pub y: f32,
}
#[derive(Serialize, Deserialize)]
pub struct Particle {
    #[serde(with = "Vec2Def")]
    pub pos: Vec2,
    #[serde(with = "Vec2Def")]
    pub vel: Vec2,
    pub material: MaterialID,
    pub lifetime: f32,
}
