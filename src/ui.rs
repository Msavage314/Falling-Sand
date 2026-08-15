use crate::Grid;
use crate::materials::MaterialID;
use egui_macroquad::egui;
use macroquad::prelude::*;
use std::collections::HashMap;
use strum::IntoEnumIterator;
pub struct UiState {
    pub active: MaterialID,
    pub radius: i32,
    pub playing: bool,
    cached_total: i32,
    cached_counts: HashMap<MaterialID, i32>,
    draw_chunk_debug: bool,
    fps: i32,
}
impl UiState {
    pub fn new() -> Self {
        return UiState {
            active: MaterialID::Sand,
            radius: 1,
            playing: true,
            cached_total: 0,
            cached_counts: HashMap::new(),
            draw_chunk_debug: true,
            fps: 0,
        };
    }
    pub fn refresh_cache(&mut self, grid: &mut Grid) {
        self.cached_total = grid.total_alive();
        self.cached_counts = grid.count_by_material();
        self.fps = get_fps()
    }

    pub fn draw(&mut self, grid: &mut Grid) -> bool {
        let mut wants_pointer = false;
        let (_rx, _ry, rw, _rh) = compute_grid_dest_rect(grid.width, grid.height);
        let pixel_scale = rw / grid.width as f32;
        let (mx, my) = mouse_position();

        draw_circle_lines(mx, my, self.radius as f32 * pixel_scale, 1.5, WHITE);

        egui_macroquad::ui(|egui_ctx| {
            wants_pointer = egui_ctx.wants_pointer_input();

            if let Some((_panel_x, _panel_y, panel_width, _panel_height)) =
                compute_side_panel_rect(grid.width, grid.height)
            {
                self.draw_materials_panel(egui_ctx, panel_width);
                self.draw_info_panel(egui_ctx, panel_width, grid);
                self.draw_controls_window(egui_ctx, grid);
                self.draw_current_window(egui_ctx, grid);
            }
        });
        if self.draw_chunk_debug {
            self.draw_chunk_debug(grid);
        }

        return wants_pointer;
    }

    fn draw_materials_panel(&mut self, egui_ctx: &egui::Context, panel_width: f32) {
        egui::SidePanel::right("Materials")
            .exact_width(panel_width)
            .show(egui_ctx, |ui| {
                ui.heading("Materials");
                ui.separator();

                for id in MaterialID::iter() {
                    let selected = self.active == id;
                    let c = (id.properties().color)(0, 0);

                    let color32 = egui::Color32::from_rgb(
                        (c.r * 255.0) as u8,
                        (c.g * 255.0) as u8,
                        (c.b * 255.0) as u8,
                    );

                    ui.horizontal(|ui| {
                        let (rect, _response) =
                            ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 2.0, color32);
                        if ui.selectable_label(selected, format!("{:?}", id)).clicked() {
                            self.active = id;
                        }
                    });
                }
            });
    }

    fn draw_info_panel(&mut self, egui_ctx: &egui::Context, panel_width: f32, grid: &mut Grid) {
        egui::SidePanel::left("Info")
            .exact_width(panel_width)
            .show(egui_ctx, |ui| {
                ui.heading("Debug");
                ui.label(format!("FPS: {}", self.fps));
                ui.separator();
                ui.label(format!("Total cells alive = {}", self.cached_total));

                let counts = &self.cached_counts;
                for id in MaterialID::iter() {
                    if id != MaterialID::Empty {
                        let count = counts.get(&id).copied().unwrap_or(0);
                        ui.label(format!("{:?}: {}", id, count));
                    }
                }
                ui.separator();
                ui.checkbox(&mut self.draw_chunk_debug, "Chunk Debug");
                ui.heading("Config");
                ui.separator();
                ui.label("Border Material");
                ui.horizontal_wrapped(|ui| {
                    for id in crate::config::BORDER_OPTIONS {
                        let selected = grid.border == id;
                        if ui.selectable_label(selected, format!("{:?}", id)).clicked() {
                            grid.border = id
                        }
                    }
                });
                ui.heading("Save");
                ui.separator();
                if ui.button("Save to file").clicked() {
                    if let Some(path) = rfd::FileDialog::new().save_file() {
                        grid.save_to_file(path.to_str().unwrap()).ok();
                    }
                }
                if ui.button("Load from file").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        grid.load_from_file(path.to_str().unwrap()).ok();
                    }
                }
            });
    }

    fn draw_controls_window(&mut self, egui_ctx: &egui::Context, grid: &mut Grid) {
        egui::Window::new("controls").show(egui_ctx, |ui| {
            let button_text = if self.playing {
                "⏸ Pause"
            } else {
                "▶ Play"
            };
            if ui.button(button_text).clicked() {
                self.playing = !self.playing;
            }
            if ui.button("X Clear").clicked() {
                grid.clear();
            }
        });
    }
    fn draw_current_window(&mut self, egui_ctx: &egui::Context, grid: &Grid) {
        egui::Window::new("Current").show(egui_ctx, |ui| {
            let (mx, my) = mouse_position();
            let (gx, gy) = screen_to_grid(grid.width, grid.height, mx, my);
            ui.label(format!("Current Material: {:?}", grid.get(gx, gy).material));
            ui.label(format!("Current Stain: {:?}", grid.get(gx, gy).stain));
            ui.label(format!("Current Status: {:?}", grid.get(gx, gy).awake));
            ui.label(format!("Current Position: ({:?},{:?})", gx, gy))
        });
    }
    fn draw_chunk_debug(&mut self, grid: &mut Grid) {
        let (rx, ry, rw, rh) = compute_grid_dest_rect(grid.width, grid.height);
        let scale_x = rw / grid.width as f32;
        let scale_y = rh / grid.height as f32;

        for cy in 0..grid.chunks_y {
            for cx in 0..grid.chunks_x {
                let idx = cy * grid.chunks_x + cx;
                let active = grid.chunks_need_update[idx];

                let px = rx + (cx * grid.chunk_size) as f32 * scale_x;
                let py = ry + (cy * grid.chunk_size) as f32 * scale_y;

                let pw = grid.chunk_size as f32 * scale_x;
                let ph = grid.chunk_size as f32 * scale_y;

                let color = if active {
                    Color::new(1.0, 0.2, 0.2, 0.5)
                } else {
                    Color::new(0.2, 0.2, 0.2, 0.15)
                };
                draw_rectangle_lines(px, py, pw, ph, 10.0, color);
            }
        }
    }
}
pub fn compute_grid_dest_rect(width: usize, height: usize) -> (f32, f32, f32, f32) {
    let grid_aspect = width as f32 / height as f32;
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
pub fn compute_side_panel_rect(width: usize, height: usize) -> Option<(f32, f32, f32, f32)> {
    let (grid_rx, _grid_ry, grid_rw, _grid_rh) = compute_grid_dest_rect(width, height);
    // Space to the right of the grid
    let panel_x = grid_rx + grid_rw;
    let panel_w = screen_width() - panel_x;

    if panel_w < 40.0 {
        return None; // No meaningful space
    }
    Some((panel_x, 0.0, panel_w, screen_height()))
}
pub fn screen_to_grid(width: usize, height: usize, screen_x: f32, screen_y: f32) -> (i32, i32) {
    let (rx, ry, rw, rh) = compute_grid_dest_rect(width, height);
    let grid_x = ((screen_x - rx) / rw * width as f32) as i32;
    let grid_y = ((screen_y - ry) / rh * height as f32) as i32;
    (grid_x, grid_y)
}
