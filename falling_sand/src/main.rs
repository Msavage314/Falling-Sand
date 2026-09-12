/// 5 different libraries are used in order to render both the pixel display and the ui.
/// They are as follows:
/// - winit - Controls events (mouse/keyboard) and window resizing.
/// - pixels - a `wgpu::Device`/`wgpu::Queue`/`wgpu::Surface` triple. stores rendered simulation image
/// - egui (core) - pure UI logic, e.g. panels, text, labels, buttons etc.
/// - egui-winit - translates winit `WindowEvents` into egui's input format and egui's output (cursor icon, clipboard) back into winit
/// - egui-wgpu - takes egui's output and renders it to the screen.
///
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

use crate::ui::UiState;
use core::time::Duration;
use materials::MaterialID;
use pixels::{Pixels, SurfaceTexture};
use sand_render::framework::Framework;
use simulation::Grid;
use std::sync::Arc;
use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

/// Stores the whole falling sand simulation and rendering information
struct App {
    /// The window to draw onto
    window: Option<Arc<Window>>,
    /// a 2d pixel buffer that will be written onto from `render`
    pixels: Option<Pixels<'static>>,
    framework: Option<Framework>,
    /// stores the simulation Grid and contains update code
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
    // fps tracking
    fps_window_start: Instant,
    fps_frames_this_window: u32,
    bloom_buffer: Vec<u8>,
    bloom_effect: Option<sand_render::bloom::BloomEffect>,
}
impl App {
    fn new() -> Self {
        let ui = UiState::new();
        let grid = Grid::new(
            config::WIDTH,
            config::HEIGHT,
            MaterialID::DenseRock,
            config::CHUNK_SIZE,
        );

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
            fps_window_start: Instant::now(),
            fps_frames_this_window: 0,
            bloom_buffer: vec![0u8; config::WIDTH * config::HEIGHT * 4],
            bloom_effect: None,
        };
    }
    fn redraw(&mut self) {
        let (Some(window), Some(pixels), Some(framework)) =
            (&self.window, &mut self.pixels, &mut self.framework)
        else {
            return;
        };
        // fps measurement
        self.fps_frames_this_window += 1;
        let elapsed = self.fps_window_start.elapsed();
        if elapsed.as_secs_f32() >= 1.0 {
            let fps = (self.fps_frames_this_window as f32 / elapsed.as_secs_f32()).round() as i32;
            self.ui.set_fps(fps);
            self.fps_frames_this_window = 0;
            self.fps_window_start = Instant::now();
        }

        // update simulation and draw mouse
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

        render::draw(
            pixels.frame_mut(),
            &mut self.bloom_buffer,
            &self.grid,
            self.ui.draw_bloom,
        );

        if let Some(bloom) = &self.bloom_effect {
            bloom.upload(pixels.queue(), &self.bloom_buffer);
        }

        let ui = &mut self.ui;
        let grid = &mut self.grid;
        framework.prepare(window, |ctx| {
            ui.draw(ctx, grid);
        });
        self.egui_wants_pointer = framework.ctx().egui_wants_pointer_input();

        let window_size = window.inner_size();
        let screen_width = window_size.width as f32;
        let screen_height = window_size.height as f32;

        let grid_width = config::WIDTH as f32;
        let grid_height = config::HEIGHT as f32;

        let grid_aspect = grid_width / grid_height;
        let screen_aspect = screen_width / screen_height;

        let (viewport_width, viewport_height) = if screen_aspect > grid_aspect {
            // Window is wider than the simulation.
            let height = screen_height;
            let width = height * grid_aspect;
            (width, height)
        } else {
            // Window is taller/narrower than the simulation.
            let width = screen_width;
            let height = width / grid_aspect;
            (width, height)
        };

        let viewport_x = (screen_width - viewport_width) * 0.5;
        let viewport_y = (screen_height - viewport_height) * 0.5;

        let bloom_viewport = (viewport_x, viewport_y, viewport_width, viewport_height);

        let render_result = pixels.render_with(|encoder, render_target, context| {
            // Normal game image.
            context.scaling_renderer.render(encoder, render_target);

            // Bloom is composited into exactly the same rectangle.
            if let Some(bloom) = &self.bloom_effect {
                bloom.render(encoder, render_target, bloom_viewport);
            }

            // UI stays on top.
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
        let framework = Framework::new(&window, size.width, size.height, scale_factor, &pixels);

        let surface_format = pixels.render_texture_format();
        self.bloom_effect = Some(sand_render::bloom::BloomEffect::new(
            pixels.device(),
            surface_format,
            config::WIDTH as u32,
            config::HEIGHT as u32,
        ));

        self.framework = Some(framework);
        self.pixels = Some(pixels);
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        // Let egui have th events first. any it doesn't use relate to the sand simulation
        if let (Some(window), Some(framework)) = (&self.window, &mut self.framework) {
            let _consumed = framework.handle_event(window, &event);
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
