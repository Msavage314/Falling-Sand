use crate::materials::MaterialID;
use crate::rng;
use crate::simulation::Grid;
use crate::stains::Stain;
pub trait Explosion: std::fmt::Debug {
    fn detonate(&self, grid: &mut Grid, x: i32, y: i32);
}
#[derive(Debug, Copy, Clone)]
pub struct CircleExplosion {
    pub radius: i32,
}

impl Explosion for CircleExplosion {
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
                            macroquad::rand::gen_range(-5.0, 5.0),
                            macroquad::rand::gen_range(-5.0, 5.0),
                            MaterialID::Fire,
                        )
                    }
                }
            }
        }
    }
}
