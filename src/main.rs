pub mod cell;
pub mod config;
pub mod explosion;
pub mod materials;
pub mod particle;
pub mod reaction;
pub mod render;
pub mod rng;
pub mod simulation;
pub mod stains;
pub mod ui;
use core::time::Duration;
use std::time::Instant;

use macroquad::prelude::*;
use materials::MaterialID;
use simulation::Grid;

fn window_config() -> Conf {
    Conf {
        window_title: String::from("Falling Sand"),
        window_width: 800,
        window_height: 600,
        window_resizable: true,
        icon: None,
        platform: miniquad::conf::Platform {
            swap_interval: Some(0),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[macroquad::main(window_config)]
async fn main() {
    let mut g = Grid::new(
        config::WIDTH,
        config::HEIGHT,
        MaterialID::DenseRock,
        config::CHUNK_SIZE,
    );
    let mut render = render::GridRenderer::new(g.width, g.height);
    let mut ui = ui::UiState::new();
    let mut frame_count = 0;

    let fps_target = config::FPS_TARGET;
    let frame_dur = Duration::from_secs_f64(1.0 / fps_target);
    let mut next_tick = Instant::now();
    loop {
        clear_background(BLACK);

        let (_, scroll_y) = mouse_wheel();
        if scroll_y != 0.0 {
            ui.radius = (ui.radius + scroll_y.signum() as i32).clamp(1, 50);
        }

        frame_count += 1;
        if ui.playing {
            g.update(frame_count % 2 == 0);
        }

        render.draw(&g);

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
                g.draw_brush(gx, gy, ui.radius, MaterialID::Empty);
            }
        }
        if is_key_pressed(KeyCode::Space) {
            ui.playing = !ui.playing
        }
        if is_key_pressed(KeyCode::Right) {
            g.update(frame_count % 2 == 0);
        }

        egui_macroquad::draw();
        next_tick += frame_dur;
        let now = Instant::now();
        if next_tick > now {
            std::thread::sleep(next_tick - now);
        } else {
            next_tick = now;
        }

        next_frame().await;
    }
}
