use crate::materials::MaterialID;
use crate::rng;
use crate::simulation::Grid;
use crate::stains::Stain;
pub trait Explosion: std::fmt::Debug {
    fn detonate(&self, grid: &mut Grid, x: i32, y: i32);
}
#[derive(Debug, Copy, Clone)]
pub struct FireExplosion {
    pub radius: i32,
    pub particle_chance: f32,
    pub velocity: f32,
}

impl Explosion for FireExplosion {
    fn detonate(&self, grid: &mut Grid, x: i32, y: i32) {
        let radius = self.radius;
        let r2 = radius * radius;

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= r2 {
                    if rng::chance(self.particle_chance) {
                        grid.add_particle(
                            x + dx,
                            y + dy,
                            macroquad::rand::gen_range(-self.velocity, self.velocity),
                            macroquad::rand::gen_range(-self.velocity, self.velocity),
                            MaterialID::Fire,
                        )
                    }
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct AcidExplosion {
    pub radius: i32,
}

impl Explosion for AcidExplosion {
    fn detonate(&self, grid: &mut Grid, x: i32, y: i32) {
        let radius = self.radius;
        let r2 = radius * radius;

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= r2 {
                    if rng::chance(0.1) {
                        grid.add_particle(
                            x + dx,
                            y + dy,
                            macroquad::rand::gen_range(-1.0, 1.0),
                            macroquad::rand::gen_range(-1.0, 1.0),
                            MaterialID::Fire,
                        )
                    }
                    if rng::chance(0.1) {
                        grid.add_particle(
                            x + dx,
                            y + dy,
                            macroquad::rand::gen_range(-5.0, 5.0),
                            macroquad::rand::gen_range(-5.0, 5.0),
                            MaterialID::Acid,
                        )
                    }
                }
            }
        }
    }
}
