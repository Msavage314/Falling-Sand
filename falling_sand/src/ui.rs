use crate::Grid;
use crate::cell::{Cell, Color};
use crate::materials::MaterialID;
use egui;
use std::collections::HashMap;
use strum::IntoEnumIterator;

/// Stores the UI structure and state, but doesn't actually contain information to render it.
///
/// # Arguments
/// - `active` - The currently active material, which will be drawn when the user clicks.
/// - `radius` - The current radius of the brush the user is drawing with.
/// - `playing` - Whether the simulation is currently playing.
/// - `cached_total` - stores the total number of pixels alive currently. stores a cached value to improve performance
/// - `cached_counts` - HashMap of how many pixels of each material are alive currently
/// - `draw_chunk_debug` - Whether or not to draw the chunk debug overlay (currently broken)
/// - `draw_marching_squares` - Whether or not to draw the marching squares overlay (currently broken)
/// - `fps` - Current simulation fps. What will be displayed to the the fps counter debug info
/// - `screen_width` - the width of the window that is being rendered into
/// - `screen_height` - the height of the window that is being rendered into
pub struct UiState {
    pub active: MaterialID,
    pub radius: i32,
    pub playing: bool,
    cached_total: i32,
    cached_counts: HashMap<MaterialID, i32>,
    draw_chunk_debug: bool,
    draw_marching_squares: bool,
    pub draw_bloom: bool,
    fps: i32,
    pub screen_width: f32,
    pub screen_height: f32,
    pub current_cell: Option<Cell>,
}
impl UiState {
    pub fn new() -> Self {
        return UiState {
            active: MaterialID::Sand,
            radius: 1,
            playing: true,
            cached_total: 0,
            cached_counts: HashMap::new(),
            draw_chunk_debug: false,
            draw_marching_squares: false,
            draw_bloom: true,
            fps: 0,
            screen_width: 0.0,
            screen_height: 0.0,
            current_cell: None,
        };
    }
    /// Refresh the cached material and fps values.
    pub fn refresh_cache(&mut self, grid: &mut Grid) {
        self.cached_total = grid.total_alive();
        self.cached_counts = grid.count_by_material();
    }
    pub fn set_fps(&mut self, fps: i32) {
        self.fps = fps;
    }
    /// describes the ui to the egui::Ui context. calls the other draw functions
    ///
    /// # UI contents
    /// - Materials panel - allows you to select different materials
    /// - info panel - shows current fps, and current material counts
    /// - controls - floating window with things like play, pause and clear
    /// - current - shows what is currently underneath the mouse cursor
    pub fn draw(&mut self, ui: &mut egui::Ui, grid: &mut Grid) {
        let (_rx, _ry, _rw, _rh) = self.compute_grid_dest_rect(grid.width, grid.height);
        let panel_width = 200.0;
        self.draw_materials_panel(ui, panel_width);
        self.draw_info_panel(ui, panel_width, grid);
        self.draw_controls_window(ui, grid);
        self.draw_current_window(ui, grid);
    }

    fn draw_materials_panel(&mut self, ui: &mut egui::Ui, panel_width: f32) {
        egui::Panel::right("Materials")
            .exact_size(panel_width)
            .show_inside(ui, |ui| {
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

    fn draw_info_panel(&mut self, ui: &mut egui::Ui, panel_width: f32, grid: &mut Grid) {
        egui::Panel::left("Info")
            .exact_size(panel_width)
            .show_inside(ui, |ui| {
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
                // ui.label(format!(
                //     "{}",
                //     crate::marching_squares::generate_lines(grid).iter().len()
                // ));
                ui.separator();
                ui.checkbox(&mut self.draw_chunk_debug, "Chunk Debug");
                ui.checkbox(&mut self.draw_marching_squares, "Marching Squares Debug");
                ui.checkbox(&mut self.draw_bloom, "Draw bloom effects");
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

    fn draw_controls_window(&mut self, ui: &egui::Context, grid: &mut Grid) {
        egui::Window::new("controls").show(ui, |ui| {
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
    fn draw_current_window(&mut self, ui: &egui::Ui, grid: &Grid) {
        egui::Window::new("Current").show(ui, |ui| {
            if let Some(cell) = self.current_cell {
                ui.label(format!("Current Material: {:?}", cell.material));
                ui.label(format!("Current Stain: {:?}", cell.stain));
                ui.label(format!("Current Status: {:?}", cell.awake));
            } else {
                ui.label("No current cell");
            };
        });
    }
    fn _draw_chunk_debug(&mut self, grid: &mut Grid) {
        let (rx, ry, rw, rh) = self.compute_grid_dest_rect(grid.width, grid.height);
        let scale_x = rw / grid.width as f32;
        let scale_y = rh / grid.height as f32;

        for cy in 0..grid.chunks_y {
            for cx in 0..grid.chunks_x {
                let idx = cy * grid.chunks_x + cx;
                let active = grid.chunks_need_update[idx];

                let _px = rx + (cx * grid.chunk_size) as f32 * scale_x;
                let _py = ry + (cy * grid.chunk_size) as f32 * scale_y;

                let _pw = grid.chunk_size as f32 * scale_x;
                let _ph = grid.chunk_size as f32 * scale_y;

                let _color = if active {
                    Color::new(1.0, 0.2, 0.2, 0.5)
                } else {
                    Color::new(0.2, 0.2, 0.2, 0.15)
                };
                //draw_rectangle_lines(px, py, pw, ph, 10.0, color);
            }
        }
    }
    fn _draw_polygons(&mut self, grid: &Grid) {
        let polygons = crate::marching_squares::stitch_polygons(
            &crate::marching_squares::generate_lines(grid),
        );
        let (rx, ry, rw, rh) = self.compute_grid_dest_rect(grid.width, grid.height);

        let scale_x = rw / grid.width as f32;
        let scale_y = rh / grid.height as f32;
        for polygon in polygons {
            for (a, b) in polygon
                .points
                .windows(2)
                .map(|w| (&w[0], &w[1]))
                .chain(std::iter::once((
                    polygon.points.last().unwrap(),
                    polygon.points.first().unwrap(),
                )))
            {
                let _x1 = rx + (a.0 + 0.5) * scale_x;
                let _y1 = ry + (a.1 + 0.5) * scale_y;

                let _x2 = rx + (b.0 + 0.5) * scale_x;
                let _y2 = ry + (b.1 + 0.5) * scale_y;

                //draw_line(x1, y1, x2, y2, 2.0, RED);
            }
        }
    }
    fn _draw_marching_squares(&mut self, grid: &Grid) {
        let segments = crate::marching_squares::generate_lines(grid);
        let (rx, ry, rw, rh) = self.compute_grid_dest_rect(grid.width, grid.height);

        let scale_x = rw / grid.width as f32;
        let scale_y = rh / grid.height as f32;

        for segment in segments {
            let _x1 = rx + (segment.start.0 + 0.5) * scale_x;
            let _y1 = ry + (segment.start.1 + 0.5) * scale_y;

            let _x2 = rx + (segment.end.0 + 0.5) * scale_x;
            let _y2 = ry + (segment.end.1 + 0.5) * scale_y;

            // draw_line(x1, y1, x2, y2, 2.0, BLUE);
        }
    }
    // fn draw_triangulation(&mut self, grid: &Grid) {
    //     let segments = crate::marching_squares::generate_lines(grid);
    //     let polygons = crate::marching_squares::stitch_polygons(&segments);
    //     let triangulations = crate::marching_squares::triangulate(&polygons);

    //     let (rx, ry, rw, rh) = self.compute_grid_dest_rect(grid.width, grid.height);
    //     let scale_x = rw / grid.width as f32;
    //     let scale_y = rh / grid.height as f32;

    //     let to_screen = |p: [f64; 2]| -> Vec2 {
    //         Vec2(
    //             rx + (p[0] as f32 + 0.5) * scale_x,
    //             ry + (p[1] as f32 + 0.5) * scale_y,
    //         )
    //     };
    //     for triangulation in &triangulations {
    //         for tri in triangulation.indices.chunks(3) {
    //             let a = to_screen(triangulation.points[tri[0] as usize]);
    //             let b = to_screen(triangulation.points[tri[1] as usize]);
    //             let c = to_screen(triangulation.points[tri[2] as usize]);
    //         }
    //     }
    // }

    pub fn compute_grid_dest_rect(&self, width: usize, height: usize) -> (f32, f32, f32, f32) {
        let grid_aspect = width as f32 / height as f32;
        let screen_aspect = self.screen_width / self.screen_height;

        let (w, h) = if screen_aspect > grid_aspect {
            let h = self.screen_height;
            let w = h * grid_aspect;
            (w, h)
        } else {
            let w = self.screen_width;
            let h = w / grid_aspect;
            (w, h)
        };

        let x = (self.screen_width - w) * 0.5;
        let y = (self.screen_height - h) * 0.5;
        (x, y, w, h)
    }
    pub fn compute_side_panel_rect(
        &self,
        width: usize,
        height: usize,
        _screen_w: f32,
        screen_h: f32,
    ) -> Option<(f32, f32, f32, f32)> {
        let (grid_rx, _grid_ry, grid_rw, _grid_rh) = self.compute_grid_dest_rect(width, height);
        // Space to the right of the grid
        let panel_x = grid_rx + grid_rw;
        let panel_w = self.screen_width - panel_x;

        if panel_w < 40.0 {
            return None; // No meaningful space
        }
        Some((panel_x, 0.0, panel_w, screen_h))
    }

    pub fn screen_to_grid(
        &self,
        width: usize,
        height: usize,
        screen_x: f32,
        screen_y: f32,
    ) -> (i32, i32) {
        let (rx, ry, rw, rh) = self.compute_grid_dest_rect(width, height);
        let grid_x = ((screen_x - rx) / rw * width as f32) as i32;
        let grid_y = ((screen_y - ry) / rh * height as f32) as i32;
        (grid_x, grid_y)
    }
}
