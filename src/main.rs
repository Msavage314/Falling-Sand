mod cell;
mod materials;
mod reaction;
mod stains;

use egui_macroquad::egui;
use macroquad::prelude::*;
use materials::Behavior;
use materials::MaterialID;
use reaction::Product;
use reaction::REACTIONS;
use stains::Stain;
use stains::StainKind;
use std::collections::HashMap;
use strum::IntoEnumIterator;

pub struct Grid {
    pub width: usize,
    pub height: usize,
    cells: Vec<MaterialID>,
    stains: Vec<Option<Stain>>,
    image: Image,
    texture: Texture2D,
    updated: Vec<bool>,
}
impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        let image = Image::gen_image_color(width as u16, height as u16, BLACK);
        let texture = Texture2D::from_image(&image);
        texture.set_filter(FilterMode::Nearest);

        return Grid {
            width: width,
            height: height,
            cells: vec![MaterialID::Empty; width * height],
            stains: vec![None; width * height],
            image,
            texture,
            updated: vec![false; width * height],
        };
    }

    pub fn get(&self, x: i32, y: i32) -> MaterialID {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return MaterialID::DenseRock;
        }

        return self.cells[y as usize * self.width + x as usize];
    }

    pub fn set(&mut self, x: i32, y: i32, value: MaterialID) {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return;
        }
        self.cells[y as usize * self.width + x as usize] = value
    }

    pub fn get_stain(&self, x: i32, y: i32) -> Option<Stain> {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return None;
        }
        return self.stains[y as usize * self.width + x as usize];
    }

    pub fn set_stain(&mut self, x: i32, y: i32, value: Option<Stain>) {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return;
        }
        self.stains[y as usize * self.width + x as usize] = value
    }

    fn can_density_swap(&mut self, cell_a: MaterialID, cell_b: MaterialID) -> bool {
        if cell_a.properties().behavior.contains(Behavior::STATIC)
            || cell_b.properties().behavior.contains(Behavior::STATIC)
        {
            return false;
        }
        if cell_a != MaterialID::Empty && cell_b != MaterialID::Empty {
            if !(cell_a.properties().behavior.contains(Behavior::FLOWS))
                && !(cell_b.properties().behavior.contains(Behavior::FLOWS))
            {
                return false;
            }
        }

        let diff = cell_a.properties().density - cell_b.properties().density;

        if diff <= 0.0 {
            return false;
        }

        let chance = diff / cell_a.properties().density.clamp(0.0, 1.0);

        return macroquad::rand::gen_range(0.0, 1.0) < chance;
    }

    fn try_flow(&mut self, cell: MaterialID, x: i32, y: i32, dir: i32, max_dist: i32) -> bool {
        let mut target = None;

        for d in 1..=max_dist {
            let nx = x + dir * d;
            let other = self.get(nx, y);

            if self.can_density_swap(cell, other) {
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

    fn mark_updated(&mut self, x: i32, y: i32) {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return;
        }
        let idx = y as usize * self.width + x as usize;
        self.updated[idx] = true
    }

    fn swap_cells(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        let a = self.get(x1, y1);
        let b = self.get(x2, y2);
        let stain_a = self.get_stain(x1, y1);
        let stain_b = self.get_stain(x2, y2);

        self.set(x1, y1, b);
        self.set(x2, y2, a);
        self.set_stain(x1, y1, stain_b);
        self.set_stain(x2, y2, stain_a);

        self.mark_updated(x1, y1);
        self.mark_updated(x2, y2);
    }

    fn get_neighbors(&mut self, x: i32, y: i32) -> [(i32, i32); 4] {
        let neighbors = [(0, -1), (1, 0), (0, 1), (-1, 0)];
        return neighbors.map(|c| (c.0 + x, c.1 + y));
    }

    fn update_stain(&mut self, x: i32, y: i32) {
        let Some(mut stain) = self.get_stain(x, y) else {
            return;
        };

        stain.timer -= get_frame_time();
        if stain.timer <= 0.0 {
            self.set_stain(x, y, None);
            if stain.kind == StainKind::Burning {
                self.set(x, y, MaterialID::Empty);
            }
        } else {
            self.set_stain(x, y, Some(stain));
        }
    }
    fn apply_product(&mut self, x: i32, y: i32, product: Product) -> bool {
        match product {
            Product::Material(mat) => {
                self.set(x, y, mat);
                true
            }
            Product::Stain(stain) => {
                self.set_stain(x, y, Some(stain));
                true
            }
            Product::NoChange => false,
        }
    }

    fn update_cell(&mut self, x: i32, y: i32) {
        let idx = y as usize * self.width + x as usize;
        if self.updated[idx] {
            return;
        }
        let cell = self.get(x, y);
        let stain = self.get_stain(x, y);
        let properties = cell.properties();

        for (cx, cy) in self.get_neighbors(x, y) {
            let other = self.get(cx, cy);
            let other_stain = self.get_stain(cx, cy);
            for reaction in REACTIONS {
                if reaction.a.matches(cell, stain)
                    && reaction.b.matches(other, other_stain)
                    && macroquad::rand::gen_range(0.0, 1.0) < reaction.chance
                {
                    let mut changed = false;
                    if let Some(oa) = reaction.output_a {
                        changed |= self.apply_product(
                            x,
                            y,
                            ((oa.apply_fn)(cell, other, stain, other_stain)),
                        )
                    }
                    if let Some(ob) = reaction.output_b {
                        changed |= self.apply_product(
                            cx,
                            cy,
                            (ob.apply_fn)(cell, other, stain, other_stain),
                        );
                    }

                    if changed {
                        self.mark_updated(x, y);
                        self.mark_updated(cx, cy);
                        return;
                    }
                }
            }
        }

        if properties.behavior.contains(Behavior::FALLS) {
            if self.can_density_swap(self.get(x, y), self.get(x, y + 1)) {
                self.swap_cells(x, y, x, y + 1);
                return;
            }
        }
        if properties.behavior.contains(Behavior::RISES) {
            if self.can_density_swap(self.get(x, y), self.get(x, y - 1)) {
                self.swap_cells(x, y, x, y - 1);
                return;
            }
        }
        if properties.behavior.contains(Behavior::GRANULAR) {
            if macroquad::rand::gen_range(0, 2) == 1 {
                let other = self.get(x + 1, y + 1);
                if self.can_density_swap(cell, other) {
                    self.swap_cells(x, y, x + 1, y + 1);
                    return;
                }
                let other = self.get(x - 1, y + 1);
                if self.can_density_swap(cell, other) {
                    self.swap_cells(x, y, x - 1, y + 1);
                    return;
                }
            } else {
                let other = self.get(x - 1, y + 1);
                if self.can_density_swap(cell, other) {
                    self.swap_cells(x, y, x - 1, y + 1);
                    return;
                }
                let other = self.get(x + 1, y + 1);
                if self.can_density_swap(cell, other) {
                    self.swap_cells(x, y, x + 1, y + 1);
                    return;
                }
            }
        }
        if properties.behavior.contains(Behavior::FLOWS) {
            let flow = macroquad::rand::gen_range(0, properties.flow_distance * 2) as i32;
            let left_first = macroquad::rand::gen_range(0, 2) == 0;

            if left_first {
                if self.try_flow(cell, x, y, -1, flow) {
                    return;
                }
                if self.try_flow(cell, x, y, 1, flow) {
                    return;
                }
            } else {
                if self.try_flow(cell, x, y, 1, flow) {
                    return;
                }
                if self.try_flow(cell, x, y, -1, flow) {
                    return;
                }
            }
        }

        // if !properties.behavior.contains(Behavior::STATIC) {
        //     let cell_b = self.get(x, y + 1);
        //     if self.get(x, y).properties().density > self.get(x, y + 1).properties().density {
        //         self.set(x, y, cell_b);
        //         self.set(x, y + 1, cell);
        //         return;
        //     }
        // }
    }

    pub fn update(&mut self, left: bool) {
        self.updated.fill(false);

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

    pub fn update_texture(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let id = self.cells[y * self.width + x];
                let base_color = id.properties().color;
                let final_color = match self.get_stain(x as i32, y as i32) {
                    Some(stain) if stain.kind == StainKind::Burning => {
                        // flicker between orange/red based on intensity, blended with base
                        Color::new(
                            base_color.r * 0.3 + 0.95 * 0.7,
                            base_color.g * 0.3 + 0.4 * 0.7,
                            base_color.b * 0.3,
                            1.0,
                        )
                    }
                    Some(stain) if stain.kind == StainKind::Wet => {
                        let darken = 1.0 - 0.3 * stain.intensity;
                        Color::new(
                            base_color.r * darken,
                            base_color.g * darken,
                            base_color.b * darken,
                            1.0,
                        )
                    }
                    _ => base_color,
                };
                self.image.set_pixel(x as u32, y as u32, final_color);
            }
        }

        self.texture.update(&self.image);
    }

    pub fn draw(&mut self) {
        self.update_texture();

        let (x, y, w, h) = self.compute_grid_dest_rect();
        draw_texture_ex(
            &self.texture,
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(w, h)),
                ..Default::default()
            },
        )
    }

    pub fn draw_brush(&mut self, cx: i32, cy: i32, radius: i32, material: MaterialID) {
        let r2 = radius * radius;

        for y in -radius..=radius {
            for x in -radius..=radius {
                if x * x + y * y <= r2 {
                    self.set(cx + x, cy + y, material)
                }
            }
        }
    }

    pub fn total_alive(&mut self) -> i32 {
        let mut count = 0;
        for cell in &self.cells {
            if *cell != MaterialID::Empty {
                count += 1;
            }
        }
        return count;
    }

    pub fn count_by_material(&mut self) -> HashMap<MaterialID, i32> {
        let mut counts = HashMap::new();
        for cell in &self.cells {
            if *cell != MaterialID::Empty {
                *counts.entry(*cell).or_insert(0) += 1
            }
        }
        return counts;
    }
}
fn screen_to_grid(grid: &mut Grid, screen_x: f32, screen_y: f32) -> (i32, i32) {
    let (rx, ry, rw, rh) = grid.compute_grid_dest_rect();
    let grid_x = ((screen_x - rx) / rw * grid.width as f32) as i32;
    let grid_y = ((screen_y - ry) / rh * grid.height as f32) as i32;
    (grid_x, grid_y)
}

#[macroquad::main("Falling Sand")]
async fn main() {
    let mut g = Grid::new(200, 150);
    let mut frame_count = 0;
    g.set(50, 0, MaterialID::Sand);
    let mut active = MaterialID::Sand;
    let mut radius = 1;
    loop {
        clear_background(BLACK);

        if is_mouse_button_down(MouseButton::Left) {
            let (mx, my) = mouse_position();
            let (gx, gy) = screen_to_grid(&mut g, mx, my);
            g.draw_brush(gx, gy, radius, active);
        }
        if is_mouse_button_down(MouseButton::Right) {
            let (mx, my) = mouse_position();
            let (gx, gy) = screen_to_grid(&mut g, mx, my);
            g.set_stain(
                gx,
                gy,
                Some(Stain {
                    kind: StainKind::Burning,
                    intensity: 1.0,
                    timer: 3.0, // 3 seconds of burning
                }),
            )
        }
        if is_key_pressed(KeyCode::Key0) {
            active = MaterialID::Empty
        } else if is_key_pressed(KeyCode::Key1) {
            active = MaterialID::Sand
        } else if is_key_pressed(KeyCode::Key2) {
            active = MaterialID::Stone
        } else if is_key_pressed(KeyCode::Key3) {
            active = MaterialID::Water
        } else if is_key_pressed(KeyCode::Key4) {
            active = MaterialID::Slime
        } else if is_key_pressed(KeyCode::Key5) {
            active = MaterialID::Acid
        } else if is_key_pressed(KeyCode::Key6) {
            active = MaterialID::Lava
        } else if is_key_pressed(KeyCode::Key7) {
            active = MaterialID::DenseRock
        } else if is_key_pressed(KeyCode::Key8) {
            active = MaterialID::Wood
        } else if is_key_pressed(KeyCode::Key9) {
            active = MaterialID::Oil
        }

        let (_, scroll_y) = mouse_wheel();
        if scroll_y != 0.0 {
            radius = (radius + scroll_y.signum() as i32).clamp(1, 50);
        }

        if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) {
            radius += 1;
        }

        // Decrease
        if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) {
            radius = (radius - 1).max(1);
        }

        frame_count += 1;

        g.update(frame_count % 2 == 0);

        g.draw();
        let (rx, ry, rw, _rh) = g.compute_grid_dest_rect();
        let pixel_scale = rw / g.width as f32;

        let (mx, my) = mouse_position();
        draw_circle_lines(mx, my, radius as f32 * pixel_scale, 1.5, WHITE);

        // if frame_count % 100 == 0 {
        //     println!("{}", g.total_alive())
        // }

        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("Debug").show(egui_ctx, |ui| {
                ui.label(format!("FPS: {}", get_fps()));
                ui.separator();
                ui.label(format!("Total cells alive = {}", g.total_alive()));

                let counts = g.count_by_material();
                for id in MaterialID::iter() {
                    if id != MaterialID::Empty {
                        let count = counts.get(&id).copied().unwrap_or(0);
                        ui.label(format!("{:?}: {}", id, count));
                    }
                }
            });
            egui::Window::new("Materials").show(egui_ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for id in MaterialID::iter() {
                        let selected = active == id;
                        if ui.selectable_label(selected, format!("{:?}", id)).clicked() {
                            active = id;
                        }
                    }
                });
            });
        });
        egui_macroquad::draw();

        next_frame().await;
    }
}
