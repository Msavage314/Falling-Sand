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
    framework: Option<crate::ui::Framework>,
    grid: Grid,
    ui: UiState,
    frame_count: u64,
    next_tick: Instant,
    frame_dur: Duration,
    // mouse states
    cursor_pos: (f32, f32),
    left_down: bool,
    right_down: bool,
    egui_wants_pointer: bool,
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
            framework: None,
            ui,
            frame_count: 0,
            next_tick: Instant::now(),
            frame_dur: Duration::from_secs_f64(1.0 / config::FPS_TARGET),
            cursor_pos: (0.0, 0.0),
            left_down: false,
            right_down: false,
            egui_wants_pointer: false,
        };
    }
    fn redraw(&mut self) {
        let (Some(window), Some(pixels), Some(framework)) =
            (&self.window, &mut self.pixels, &mut self.framework)
        else {
            return;
        };

        // 1. tick sim + paint from mouse (gate on last frame's wants_pointer)
        let now = Instant::now();
        if now >= self.next_tick && self.ui.playing {
            self.grid.update(self.frame_count % 2 == 0);
            self.frame_count += 1;
            self.next_tick += self.frame_dur;
            if self.next_tick < now {
                self.next_tick = now + self.frame_dur;
            }
        }

        if !self.egui_wants_pointer {
            if let Ok((gx, gy)) = pixels.window_pos_to_pixel(self.cursor_pos) {
                if self.left_down {
                    self.grid
                        .draw_brush(gx as i32, gy as i32, self.ui.radius, self.ui.active);
                }
                if self.right_down {
                    self.grid
                        .draw_brush(gx as i32, gy as i32, self.ui.radius, MaterialID::Empty);
                }
            }
        }

        if self.frame_count % 15 == 0 {
            self.ui.refresh_cache(&mut self.grid);
        }

        // 2. rasterize sand into the CPU buffer
        render::draw(pixels.frame_mut(), &self.grid);

        // 3. build the egui frame
        let ui = &mut self.ui;
        let grid = &mut self.grid;
        framework.prepare(window, |ctx| {
            ui.draw(ctx, grid);
        });
        self.egui_wants_pointer = framework.ctx().wants_pointer_input();

        // 4. render: pixels' built-in upscale pass, then egui on top
        let render_result = pixels.render_with(|encoder, render_target, context| {
            context.scaling_renderer.render(encoder, render_target);
            framework.render(encoder, render_target, context);
            Ok(())
        });
        if render_result.is_err() {
            return;
        }

        window.request_redraw();
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
        let size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
        let framework = ui::Framework::new(&window, size.width, size.height, scale_factor, &pixels);
        self.framework = Some(framework);
        self.pixels = Some(pixels);
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        // Let egui have th events first. any it doesn't use relate to the sand simulation
        if let (Some(window), Some(framework)) = (&self.window, &mut self.framework) {
            let consumed = framework.handle_event(window, &event);
        }
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(pixels) = &mut self.pixels {
                    pixels.resize_surface(size.width, size.height).unwrap();
                }
                if let Some(framework) = &mut self.framework {
                    framework.resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                self.redraw();
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
