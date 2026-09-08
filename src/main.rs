pub mod cell;
pub mod config;
pub mod explosion;
pub mod marching_squares;
pub mod materials;
pub mod particle;
pub mod reaction;
pub mod render;
pub mod rng;
pub mod simulation;
pub mod stains;
pub mod ui;
use core::time::Duration;
use materials::MaterialID;
use pixels::{Pixels, SurfaceTexture};
use simulation::Grid;
use std::sync::Arc;
use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use crate::ui::UiState;
struct App {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    grid: Grid,
    ui: UiState,
    frame_count: u64,
    next_tick: Instant,
    frame_dur: Duration,
    // mouse states
    cursor_pos: (f32, f32),
    left_down: bool,
    right_down: bool,
}
impl App {
    fn new() -> Self {
        let ui = UiState::new();
        let mut grid = Grid::new(
            config::WIDTH,
            config::HEIGHT,
            MaterialID::DenseRock,
            config::CHUNK_SIZE,
        );
        grid.set_material(0, 0, MaterialID::Water);

        return Self {
            window: None,
            pixels: None,
            grid: grid,
            ui,
            frame_count: 0,
            next_tick: Instant::now(),
            frame_dur: Duration::from_secs_f64(1.0 / config::FPS_TARGET),
            cursor_pos: (0.0, 0.0),
            left_down: false,
            right_down: false,
        };
    }
}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Falling Sand")
                        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0)),
                )
                .unwrap(),
        );
        let size = window.inner_size();
        let surface_texture = SurfaceTexture::new(size.width, size.height, window.clone());
        let mut pixels =
            Pixels::new(config::WIDTH as u32, config::HEIGHT as u32, surface_texture).unwrap();
        pixels.set_scaling_mode(pixels::ScalingMode::Fill);
        self.window = Some(window);
        self.pixels = Some(pixels)
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(pixels) = &mut self.pixels {
                    pixels.resize_surface(size.width, size.height).unwrap();
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                if now >= self.next_tick {
                    self.grid.update(self.frame_count % 2 == 0);
                    self.frame_count += 1;
                    self.next_tick += self.frame_dur;
                    if self.next_tick < now {
                        self.next_tick = now + self.frame_dur; // don't spiral if we fall behind
                    }
                }
                if let Some(pixels) = &self.pixels {
                    if let Ok((gx, gy)) = pixels.window_pos_to_pixel(self.cursor_pos) {
                        if self.left_down {
                            self.grid.draw_brush(
                                gx as i32,
                                gy as i32,
                                self.ui.radius,
                                self.ui.active,
                            );
                            if self.right_down {
                                self.grid.draw_brush(
                                    gx as i32,
                                    gy as i32,
                                    self.ui.radius,
                                    MaterialID::Empty,
                                );
                            }
                        }
                    }
                }
                if let Some(pixels) = &mut self.pixels {
                    render::draw(pixels.frame_mut(), &self.grid);
                    pixels.render().unwrap();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = (position.x as f32, position.y as f32);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let pressed = state == winit::event::ElementState::Pressed;
                match button {
                    winit::event::MouseButton::Left => self.left_down = pressed,
                    winit::event::MouseButton::Right => self.right_down = pressed,
                    _ => {}
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll_y = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 20.0, // rough normalization
                };
                if scroll_y != 0.0 {
                    self.ui.radius = (self.ui.radius + scroll_y.signum() as i32).clamp(1, 50);
                }
            }

            _ => (),
        }
    }
    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Poll);
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
