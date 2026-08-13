use crate::materials::Behavior;
use crate::materials::MaterialID;
use crate::simulation::Grid;
use crate::stains::Stain;
use crate::{rng, stains};

/// Trait for implementing explosives into the falling sand simulation
///
/// # Examples
/// ```rust
/// pub struct TestExplosion {}
/// impl Explosion for TestExplosion {
///     fn detonate(&self, grid: &mut Grid, x:i32,y:i32){
///     println!("Explosive Detonated!")    
///     }
/// }
/// ```
///
///
pub trait Explosion: std::fmt::Debug {
    /// defines what should happen to the grid when this explosive detonates, as coordinates `x` and `y`
    fn detonate(&self, grid: &mut Grid, x: i32, y: i32);
}

/// Defines an explosion that simply fills the area around it with fire, and releases fire particles.
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

        let mut destroying = true;
        loop {
            let cell = grid.get(x, y);
            let resistance = cell.material.properties().explosion_resistance;
            power -= resistance;
            // the ray has used all its "energy"
            if power <= 0.0 {
                destroying = false;
            }

            if (x0 - x) * (x0 - x) + (y0 - y) * (y0 - y) > (self.radius * self.radius) {
                destroying = false;
            }
            if !destroying {
                if rng::chance(0.1) {
                    break;
                }
            }
            if grid
                .get(x, y)
                .material
                .properties()
                .behavior
                .contains(Behavior::POWDER)
                | grid
                    .get(x, y)
                    .material
                    .properties()
                    .behavior
                    .contains(Behavior::STATIC)
            {
                let distance = (((x - x0).abs() * (x - x0).abs() + (y - y0).abs() * (y - y0).abs())
                    as f32)
                    .sqrt();

                grid.set_stain(
                    x,
                    y,
                    Some(Stain {
                        kind: stains::StainKind::Charred,
                        intensity: (1.0 - (distance / (self.radius as f32 * 2.0))).min(0.7),
                        timer: 50.0,
                    }),
                );
            }
            if destroying {
                if power > 0.0 {
                    if rng::chance(0.3) {
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
                    if cell.material != MaterialID::Dynamite
                        && cell.material != MaterialID::Fire
                        && !cell
                            .material
                            .properties()
                            .behavior
                            .contains(Behavior::STATIC)
                    {
                        if rng::chance(0.9) {
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

                            grid.add_particle(x, y, 7.0 * vx, 7.0 * vy, cell.material);
                        }
                    }
                    grid.create(x, y, MaterialID::Fire);
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
        let r = self.radius * 2;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx.abs() == r || dy.abs() == r {
                    self.cast_ray(grid, x, y, x + dx, y + dy);
                }
            }
        }
    }
}
