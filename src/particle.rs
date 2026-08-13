use egui_macroquad::egui::Vec2;

use crate::materials::MaterialID;

pub struct Particle {
    pub pos: Vec2,
    pub vel: Vec2,
    pub material: MaterialID,
    pub lifetime: f32,
}
