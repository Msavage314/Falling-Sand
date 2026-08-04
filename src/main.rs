mod cell;
mod config;
mod materials;
mod reaction;
mod rng;
mod simulation;
mod stains;
mod ui;
use macroquad::prelude::*;
use materials::MaterialID;
use simulation::Grid;
use stains::Stain;
use stains::StainKind;

#[macroquad::main("Falling Sand")]
async fn main() {
    let mut g = Grid::new(200, 150, MaterialID::DenseRock);
    let mut ui = ui::UiState::new();
    let mut frame_count = 0;
    let mut playing = true;
    loop {
        clear_background(BLACK);

        let (_, scroll_y) = mouse_wheel();
        if scroll_y != 0.0 {
            ui.radius = (ui.radius + scroll_y.signum() as i32).clamp(1, 50);
        }

        frame_count += 1;
        if playing {
            g.update(frame_count % 2 == 0);
        }

        g.draw();

        if frame_count % 15 == 0 {
            ui.refresh_cache(&mut g);
        }
        let egui_wants_pointer = ui.draw(&mut g);

        // Only accept mouse input if you are clicking on something other than the ui
        if !egui_wants_pointer {
            if is_mouse_button_down(MouseButton::Left) {
                let (mx, my) = mouse_position();
                let (gx, gy) = ui::screen_to_grid(g.width, g.height, mx, my);
                g.draw_brush(gx, gy, ui.radius, ui.active);
            }
            if is_mouse_button_down(MouseButton::Right) {
                let (mx, my) = mouse_position();
                let (gx, gy) = ui::screen_to_grid(g.width, g.height, mx, my);
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
