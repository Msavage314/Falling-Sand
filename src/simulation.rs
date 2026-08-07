use crate::cell::Cell;
use crate::config::GRAVITY;
use crate::materials::Behavior;
use crate::materials::MaterialID;
use crate::materials::MaterialID::Fire;
use crate::particle::Particle;
use crate::reaction;
use crate::reaction::Product;
use crate::rng;
use crate::rng::chance;
use crate::stains::Stain;
use crate::stains::StainKind;
use egui_macroquad::egui::vec2;
use macroquad::prelude::*;
use std::collections::HashMap;
/// Owns the simulation state and rendering surface for the falling-sand grid.
///
/// Cells are stored in a flat row `Vec<Cell>` rather than a 2d array for increased performance
pub struct Grid {
    pub width: usize,
    pub height: usize,
    cells: Vec<Cell>,
    /// List of particles. They have velocity and position, and move according to physics until they
    /// collide with a non empty cell
    pub particles: Vec<Particle>,
    /// Tracks which cells have already been touched this tick, so a cell moved
    /// by an earlier update will not be updated again
    updated: Vec<bool>,
    /// Material returned for any coordinate outside the grid bounds.
    pub border: MaterialID,
}
impl Grid {
    pub fn new(width: usize, height: usize, border: MaterialID) -> Self {
        return Grid {
            width: width,
            height: height,
            cells: vec![
                Cell {
                    material: MaterialID::Empty,
                    stain: None,
                    color: (MaterialID::Empty.properties().color)(0, 0),
                    awake: true
                };
                width * height
            ],
            particles: vec![],
            updated: vec![false; width * height],
            border,
        };
    }

    pub fn get(&self, x: i32, y: i32) -> Cell {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return Cell {
                material: self.border,
                stain: None,
                color: (self.border.properties().color)(x, y),
                awake: true,
            };
        }

        return self.cells[y as usize * self.width + x as usize];
    }
    fn disturb_neighbors(&mut self, x: i32, y: i32) {
        const OFFSETS: [(i32, i32); 8] = [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ];
        for (dx, dy) in OFFSETS {
            let (nx, ny) = (x + dx, y + dy);
            if self.get(nx, ny).awake {
                continue;
            }
            let cell = self.get(nx, ny);
            if cell.material == MaterialID::Empty {
                continue;
            }
            if macroquad::rand::gen_range(0.0, 1.0) < cell.material.properties().wake_chance {
                self.cells[ny as usize * self.width + nx as usize].awake = true;
            }
        }
    }
    pub fn clear(&mut self) {
        self.cells = vec![
            Cell {
                material: MaterialID::Empty,
                stain: None,
                color: (MaterialID::Empty.properties().color)(0, 0),
                awake: true
            };
            self.width * self.height
        ]
    }

    pub fn set(&mut self, x: i32, y: i32, value: Cell) {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return;
        }

        self.cells[y as usize * self.width + x as usize] = value;
        self.cells[y as usize * self.width + x as usize].awake = true;
    }
    pub fn set_material(&mut self, x: i32, y: i32, material: MaterialID) {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return;
        }
        self.cells[y as usize * self.width + x as usize].material = material;
    }
    pub fn create(&mut self, x: i32, y: i32, material: MaterialID) {
        // create is the same as set, but is used for adding materials
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return;
        }
        self.cells[y as usize * self.width + x as usize] = Cell {
            material,
            stain: None,
            color: (material.properties().color)(x, y),
            awake: true,
        }
    }

    pub fn set_stain(&mut self, x: i32, y: i32, stain: Option<Stain>) {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return;
        }
        self.cells[y as usize * self.width + x as usize].stain = stain;
    }

    fn can_density_swap(&mut self, cell_a: Cell, cell_b: Cell, falling: bool) -> bool {
        let mat_a = cell_a.material;
        let mat_b = cell_b.material;

        if mat_a.properties().behavior.contains(Behavior::STATIC)
            || mat_b.properties().behavior.contains(Behavior::STATIC)
        {
            return false;
        }
        if mat_a != MaterialID::Empty && mat_b != MaterialID::Empty {
            if !(mat_a.properties().behavior.contains(Behavior::FLOWS))
                && !(mat_b.properties().behavior.contains(Behavior::FLOWS))
            {
                return false;
            }
        }

        let diff = mat_a.properties().density - mat_b.properties().density;

        let ok = if falling { diff > 0.0 } else { diff < 0.0 };
        if !ok {
            return false;
        }
        // Moving into empty space is pure gravity/buoyancy against nothing — always succeeds.
        if mat_b == MaterialID::Empty {
            return true;
        }
        let denom = mat_a
            .properties()
            .density
            .max(mat_b.properties().density)
            .max(0.01);
        let chance = diff.abs() / denom;

        return rng::chance(chance);
    }

    fn try_flow(
        &mut self,
        cell: Cell,
        x: i32,
        y: i32,
        dir: i32,
        max_dist: i32,
        falling: bool,
    ) -> bool {
        let mut target = None;

        for d in 1..=max_dist {
            let nx = x + dir * d;
            let other = self.get(nx, y);

            if self.can_density_swap(cell, other, falling) {
                target = Some(nx);
            } else {
                break;
            }
        }
        if let Some(nx) = target {
            self.swap_cells(x, y, nx, y);
            return true;
        } else {
            return false;
        }
    }

    pub fn mark_updated(&mut self, x: i32, y: i32) {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return;
        }
        let idx = y as usize * self.width + x as usize;
        self.updated[idx] = true
    }

    fn swap_cells(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        let a = self.get(x1, y1);
        let b = self.get(x2, y2);

        self.set(x1, y1, b);
        self.set(x2, y2, a);

        self.disturb_neighbors(x1, y1);
        self.disturb_neighbors(x2, y2);

        self.mark_updated(x1, y1);
        self.mark_updated(x2, y2);
    }

    fn get_neighbors(&mut self, x: i32, y: i32) -> [(i32, i32); 4] {
        let neighbors = [(0, -1), (1, 0), (0, 1), (-1, 0)];
        return neighbors.map(|c| (c.0 + x, c.1 + y));
    }

    fn update_stain(&mut self, x: i32, y: i32) {
        let Some(mut stain) = self.get(x, y).stain else {
            return;
        };
        if stain.kind == StainKind::Slimy {
            return;
        }
        if stain.kind == StainKind::Wet {
            stain.intensity -= 0.05 * get_frame_time(); // tune evaporation rate
            if stain.intensity <= 0.0 {
                self.set_stain(x, y, None);
                return;
            }
        }

        stain.timer -= get_frame_time();
        if stain.timer <= 0.0 {
            self.set_stain(x, y, None);
            if stain.kind == StainKind::Burning {
                self.set_material(x, y, MaterialID::Empty);
                let idx = y as usize * self.width + x as usize;
                self.cells[idx].color = (MaterialID::Empty.properties().color)(x, y)
            }
        } else {
            self.set_stain(x, y, Some(stain));
        }
    }
    fn apply_product(&mut self, x: i32, y: i32, product: Product) -> bool {
        match product {
            Product::Material(mat) => {
                let idx = y as usize * self.width + x as usize;
                if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
                    return false;
                }
                self.cells[idx].material = mat;
                if mat != MaterialID::Empty {
                    self.cells[idx].color = (mat.properties().color)(x, y);
                } else {
                    self.cells[idx].color = (mat.properties().color)(x, y)
                }
                true
            }
            Product::Stain(stain) => {
                self.set_stain(x, y, Some(stain));
                true
            }
            Product::Explosion {
                source,
                x_offset,
                y_offset,
            } => {
                source.detonate(self, x + x_offset, y + y_offset);
                true
            }
            Product::NoChange => false,
        }
    }
    fn update_particles(&mut self) {
        let mut particles = std::mem::take(&mut self.particles);

        particles.retain_mut(|particle| {
            particle.vel += vec2(0.0, GRAVITY);
            let new_pos = (particle.pos + particle.vel).floor();

            if self.get(new_pos.x as i32, new_pos.y as i32).material != MaterialID::Empty {
                self.create(
                    particle.pos.x as i32,
                    particle.pos.y as i32,
                    particle.material,
                );
                false
            } else {
                particle.pos = new_pos;
                true
            }
        });

        self.particles = particles;
    }
    pub fn add_particle(&mut self, x: i32, y: i32, vx: f32, vy: f32, material: MaterialID) {
        self.particles.push(Particle {
            pos: vec2(x as f32, y as f32),
            vel: vec2(vx, vy),
            material: material,
            lifetime: 50.0,
        })
    }

    fn update_cell(&mut self, x: i32, y: i32) {
        let idx = y as usize * self.width + x as usize;
        if self.updated[idx] {
            return;
        }
        let cell = self.get(x, y);
        let properties = cell.material.properties();

        for (cx, cy) in self.get_neighbors(x, y) {
            let other = self.get(cx, cy);
            for reaction in &reaction::REACTIONS_BY_MATERIAL[cell.material as usize] {
                if reaction.a.matches(cell)
                    && reaction.b.matches(other)
                    && rng::chance(reaction.chance)
                {
                    let mut changed = false;
                    if let Some(oa) = reaction.output_a {
                        if self.apply_product(x, y, (oa.apply_fn)(cell, other)) {
                            self.mark_updated(x, y);
                            changed |= true;
                        }
                    }
                    if let Some(ob) = reaction.output_b {
                        if self.apply_product(cx, cy, (ob.apply_fn)(cell, other)) {
                            self.mark_updated(cx, cy);
                            changed |= true;
                        }
                    }
                    if changed {
                        return;
                    }
                }
            }
        }

        if cell.behaviors().contains(Behavior::FALLS) {
            if self.can_density_swap(self.get(x, y), self.get(x, y + 1), true) {
                self.swap_cells(x, y, x, y + 1);
                return;
            }
        }
        if cell.behaviors().contains(Behavior::RISES) {
            if self.can_density_swap(self.get(x, y), self.get(x, y - 1), false) {
                self.swap_cells(x, y, x, y - 1);
                return;
            }
        }
        if cell.behaviors().contains(Behavior::GRANULAR) && cell.awake {
            if rng::chance(0.5) {
                let other = self.get(x + 1, y + 1);
                if self.can_density_swap(cell, other, true) {
                    self.swap_cells(x, y, x + 1, y + 1);
                    return;
                }
                let other = self.get(x - 1, y + 1);
                if self.can_density_swap(cell, other, true) {
                    self.swap_cells(x, y, x - 1, y + 1);
                    return;
                }
            } else {
                let other = self.get(x - 1, y + 1);
                if self.can_density_swap(cell, other, true) {
                    self.swap_cells(x, y, x - 1, y + 1);
                    return;
                }
                let other = self.get(x + 1, y + 1);
                if self.can_density_swap(cell, other, true) {
                    self.swap_cells(x, y, x + 1, y + 1);
                    return;
                }
            }
        }
        if cell.behaviors().contains(Behavior::FLOWS) {
            let flow = macroquad::rand::gen_range(0, properties.flow_distance * 2) as i32;
            let left_first = rng::chance(0.5);
            let falling = !cell.behaviors().contains(Behavior::RISES);

            if left_first {
                if self.try_flow(cell, x, y, -1, flow, falling) {
                    return;
                }
                if self.try_flow(cell, x, y, 1, flow, falling) {
                    return;
                }
            } else {
                if self.try_flow(cell, x, y, 1, flow, falling) {
                    return;
                }
                if self.try_flow(cell, x, y, -1, flow, falling) {
                    return;
                }
            }
        }

        // if we made it to here, then nothing happened this frame
        let idx = y as usize * self.width + x as usize;
        self.cells[idx].awake = false;
    }

    pub fn update(&mut self, left: bool) {
        self.updated.fill(false);
        self.update_particles();
        for y in (0..self.height).rev() {
            if left {
                for x in (0..self.width).rev() {
                    self.update_cell(x as i32, y as i32);
                    self.update_stain(x as i32, y as i32);
                }
            } else {
                for x in 0..self.width {
                    self.update_cell(x as i32, y as i32);
                    self.update_stain(x as i32, y as i32);
                }
            }
        }
    }

    fn compute_grid_dest_rect(&mut self) -> (f32, f32, f32, f32) {
        let grid_aspect = self.width as f32 / self.height as f32;
        let screen_aspect = screen_width() / screen_height();

        let (w, h) = if screen_aspect > grid_aspect {
            // window wider than grid -> letterbox left/right
            let h = screen_height();
            let w = h * grid_aspect;
            (w, h)
        } else {
            // window taller than grid -> letterbox top/bottom
            let w = screen_width();
            let h = w / grid_aspect;
            (w, h)
        };

        let x = (screen_width() - w) * 0.5;
        let y = (screen_height() - h) * 0.5;
        return (x, y, w, h);
    }

    pub fn draw_brush(&mut self, cx: i32, cy: i32, radius: i32, material: MaterialID) {
        let r2 = radius * radius;

        for y in -radius..=radius {
            for x in -radius..=radius {
                if x * x + y * y <= r2 {
                    self.create(cx + x, cy + y, material)
                }
            }
        }
    }
    pub fn draw_particles(&mut self, cx: i32, cy: i32, radius: i32, material: MaterialID) {
        let r2 = radius * radius;

        for y in -radius..=radius {
            for x in -radius..=radius {
                if x * x + y * y <= r2 {
                    self.add_particle(cx + x, cy + y, 0.0, 1.0, material)
                }
            }
        }
    }

    pub fn total_alive(&mut self) -> i32 {
        let mut count = 0;
        for cell in &self.cells {
            if (*cell).material != MaterialID::Empty {
                count += 1;
            }
        }
        return count;
    }

    pub fn count_by_material(&mut self) -> HashMap<MaterialID, i32> {
        let mut counts = HashMap::new();
        for cell in &self.cells {
            if cell.material != MaterialID::Empty {
                *counts.entry(cell.material).or_insert(0) += 1
            }
        }
        return counts;
    }
}
