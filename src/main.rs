mod cell;
mod materials;
mod reaction;
mod rng;
mod stains;

use cell::Cell;
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

/// Materials which the user is allowed to pick as the edge of the simulation
/// `DenseRock` results in sand stopping at the edge, whereas `Empty` makes it fall offscreen
const BORDER_OPTIONS: [MaterialID; 2] = [MaterialID::Empty, MaterialID::DenseRock];

/// Owns the simulation state and rendering surface for the falling-sand grid.
///
/// Cells are stored in a flat row `Vec<Cell>` rather than a 2d array for increased performance
pub struct Grid {
    pub width: usize,
    pub height: usize,
    cells: Vec<Cell>,
    image: Image,
    texture: Texture2D,
    /// Tracks which cells have already been touched this tick, so a cell moved
    /// by an earlier update will not be updated again
    updated: Vec<bool>,
    /// Material returned for any coordinate outside the grid bounds.
    border: MaterialID,
}
impl Grid {
    pub fn new(width: usize, height: usize, border: MaterialID) -> Self {
        let image = Image::gen_image_color(width as u16, height as u16, BLACK);
        let texture = Texture2D::from_image(&image);
        texture.set_filter(FilterMode::Nearest);

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
            image,
            texture,
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
            awake: false,
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
            Product::NoChange => false,
        }
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
            for reaction in REACTIONS {
                if reaction.a.matches(cell)
                    && reaction.b.matches(other)
                    && rng::chance(reaction.chance)
                {
                    let mut changed = false;
                    if let Some(oa) = reaction.output_a {
                        changed |= self.apply_product(x, y, (oa.apply_fn)(cell, other))
                    }
                    if let Some(ob) = reaction.output_b {
                        changed |= self.apply_product(cx, cy, (ob.apply_fn)(cell, other));
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
            if self.can_density_swap(self.get(x, y), self.get(x, y + 1), true) {
                self.swap_cells(x, y, x, y + 1);
                return;
            }
        }
        if properties.behavior.contains(Behavior::RISES) {
            if self.can_density_swap(self.get(x, y), self.get(x, y - 1), false) {
                self.swap_cells(x, y, x, y - 1);
                return;
            }
        }
        if properties.behavior.contains(Behavior::GRANULAR) && cell.awake {
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
            // if we made it to here, then nothing happened this frame
            let idx = y as usize * self.width + x as usize;
            self.cells[idx].awake = false;
        }
        if properties.behavior.contains(Behavior::FLOWS) {
            let flow = macroquad::rand::gen_range(0, properties.flow_distance * 2) as i32;
            let left_first = rng::chance(0.5);
            let falling = !properties.behavior.contains(Behavior::RISES);

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

                let base_color = id.color;
                let final_color = match id.stain {
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
                // if id.material != MaterialID::Empty {
                //     if id.awake {
                //         self.image
                //             .set_pixel(x as u32, y as u32, Color::new(1.0, 0.0, 0.0, 1.0));
                //     } else {
                //         self.image
                //             .set_pixel(x as u32, y as u32, Color::new(0.0, 1.0, 0.0, 1.0));
                //     }
                // } else {
                //     self.image.set_pixel(x as u32, y as u32, final_color);
                // }
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
                    self.create(cx + x, cy + y, material)
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
fn screen_to_grid(grid: &mut Grid, screen_x: f32, screen_y: f32) -> (i32, i32) {
    let (rx, ry, rw, rh) = grid.compute_grid_dest_rect();
    let grid_x = ((screen_x - rx) / rw * grid.width as f32) as i32;
    let grid_y = ((screen_y - ry) / rh * grid.height as f32) as i32;
    (grid_x, grid_y)
}
fn compute_side_panel_rect(grid_rx: f32, grid_rw: f32) -> Option<(f32, f32, f32, f32)> {
    // Space to the right of the grid
    let panel_x = grid_rx + grid_rw;
    let panel_w = screen_width() - panel_x;

    if panel_w < 40.0 {
        return None; // No meaningful space
    }
    Some((panel_x, 0.0, panel_w, screen_height()))
}

#[macroquad::main("Falling Sand")]
async fn main() {
    let mut g = Grid::new(200, 150, MaterialID::DenseRock);
    let mut frame_count = 0;
    let mut active = MaterialID::Sand;
    let mut radius = 1;
    let mut playing = true;
    loop {
        clear_background(BLACK);

        let (_, scroll_y) = mouse_wheel();
        if scroll_y != 0.0 {
            radius = (radius + scroll_y.signum() as i32).clamp(1, 50);
        }

        frame_count += 1;
        if playing {
            g.update(frame_count % 2 == 0);
        }
        g.draw();

        let (_rx, _ry, rw, _rh) = g.compute_grid_dest_rect();

        let pixel_scale = rw / g.width as f32;

        let (mx, my) = mouse_position();

        draw_circle_lines(mx, my, radius as f32 * pixel_scale, 1.5, WHITE);

        // Stores whether you have clicked on a egui window, to prevent it drawing underneath
        let mut egui_wants_pointer = false;
        let (rx, _ry, rw, _rh) = g.compute_grid_dest_rect();
        egui_macroquad::ui(|egui_ctx| {
            egui_wants_pointer = egui_ctx.wants_pointer_input();
            if let Some((_px, _pyy, pw, _ph)) = compute_side_panel_rect(rx, rw) {
                egui::SidePanel::right("Materials")
                    .exact_width(pw)
                    .show(egui_ctx, |ui| {
                        ui.heading("Materials");
                        ui.separator();

                        for id in MaterialID::iter() {
                            let selected = active == id;
                            let c = (id.properties().color)(0, 0);

                            let color32 = egui::Color32::from_rgb(
                                (c.r * 255.0) as u8,
                                (c.g * 255.0) as u8,
                                (c.b * 255.0) as u8,
                            );

                            ui.horizontal(|ui| {
                                let (rect, _response) = ui.allocate_exact_size(
                                    egui::vec2(16.0, 16.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(rect, 2.0, color32);
                                if ui.selectable_label(selected, format!("{:?}", id)).clicked() {
                                    active = id;
                                }
                            });
                        }
                    });
                egui::SidePanel::left("Info")
                    .exact_width(pw)
                    .show(egui_ctx, |ui| {
                        ui.heading("Debug");
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
                        ui.separator();
                        ui.heading("Config");
                        ui.separator();
                        ui.label("Border Material");
                        ui.horizontal_wrapped(|ui| {
                            for id in BORDER_OPTIONS {
                                let selected = g.border == id;
                                if ui.selectable_label(selected, format!("{:?}", id)).clicked() {
                                    g.border = id
                                }
                            }
                        });
                    });

                egui::Window::new("controls").show(egui_ctx, |ui| {
                    let button_text = if playing { "⏸ Pause" } else { "▶ Play" };
                    if ui.button(button_text).clicked() {
                        playing = !playing;
                    }
                });
                egui::Window::new("Current").show(egui_ctx, |ui| {
                    let (mx, my) = mouse_position();
                    let (gx, gy) = screen_to_grid(&mut g, mx, my);
                    ui.label(format!("Current Material: {:?}", g.get(gx, gy).material));
                    ui.label(format!("Current Stain: {:?}", g.get(gx, gy).stain));
                });
            }
        });
        // Only accept mouse input if you are clicking on something other than the ui
        if !egui_wants_pointer {
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
        }
        if is_key_pressed(KeyCode::Space) {
            playing = !playing
        }

        egui_macroquad::draw();

        next_frame().await;
    }
}
