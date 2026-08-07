use crate::materials::MaterialID::{self, DenseRock};
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
#[derive(Debug, Clone, Copy)]
pub struct RayTracedExplosion {
    pub radius: i32,
    pub power: f32,
}
impl RayTracedExplosion {
    fn cast_ray(&self, grid: &mut Grid, x0: i32, y0: i32, x1: i32, y1: i32) {
        let mut power = self.power;
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };

        let mut err = dx + dy;
        let mut x = x0;
        let mut y = y0;

        loop {
            // the ray has used all its "energy"
            if power <= 0.0 {
                break;
            }
            if (x0 - x) * (x0 - x) + (y0 - y) * (y0 - y) > (self.radius * self.radius) {
                break;
            }
            let cell = grid.get(x, y);
            let resistance = cell.material.properties().explosion_resistance;
            if cell.material != MaterialID::Empty {
                power -= resistance;
                if power > 0.0 {
                    grid.create(x, y, MaterialID::Fire);
                    if rng::chance(0.5) {
                        let spread = 100.0;

                        let vy = (y1 - y0) as f32;
                        let vx = (x1 - x0) as f32;
                        let vx = vx + macroquad::rand::gen_range(-spread, spread);
                        let vy = vy + macroquad::rand::gen_range(-spread, spread);
                        let len = (vx * vx + vy * vy).sqrt();

                        let (vx, vy) = if len > 0.0 {
                            (vx / len, vy / len)
                        } else {
                            (0.0, 0.0)
                        };

                        grid.add_particle(x, y, 5.0 * vx, 5.0 * vy, MaterialID::Fire);
                    }
                }
            }
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }
}
impl Explosion for RayTracedExplosion {
    fn detonate(&self, grid: &mut Grid, x: i32, y: i32) {
        let r = self.radius;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx.abs() == r || dy.abs() == r {
                    self.cast_ray(grid, x, y, x + dx, y + dy);
                }
            }
        }
    }
}
