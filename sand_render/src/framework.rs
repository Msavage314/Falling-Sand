use egui_wgpu::RendererOptions;
use pixels::{Pixels, PixelsContext, SurfaceTexture, wgpu};
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::event::WindowEvent;
use winit::window::Window;

use crate::bloom::BloomEffect;
use crate::simulation::{FrameInput, Simulation};

pub struct Framework {
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    screen_descriptor: egui_wgpu::ScreenDescriptor,
    renderer: egui_wgpu::Renderer,
    paint_jobs: Vec<egui::ClippedPrimitive>,
    textures: egui::TexturesDelta,

    pixels: Pixels<'static>,
    bloom_effect: Option<BloomEffect>,
    bloom_buffer: Vec<u8>,
    grid_width: usize,
    grid_height: usize,

    cursor_pos: (f32, f32),
    left_down: bool,
    right_down: bool,
    scroll_delta: f32,
    egui_wants_pointer: bool,

    frame_count: u64,
    next_tick: Instant,
    frame_dur: Duration,
    fps_window_start: Instant,
    fps_frames_this_window: u32,
    fps: i32,
}

impl Framework {
    pub fn new(
        window: Arc<Window>,
        grid_width: usize,
        grid_height: usize,
        tick_rate: f64,
        enable_bloom: bool,
    ) -> Self {
        let size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;

        let surface_texture = SurfaceTexture::new(size.width, size.height, window.clone());
        let mut pixels =
            Pixels::new(grid_width as u32, grid_height as u32, surface_texture).unwrap();
        pixels.set_scaling_mode(pixels::ScalingMode::Fill);

        let surface_format = pixels.render_texture_format();
        let bloom_effect = enable_bloom.then(|| {
            BloomEffect::new(
                pixels.device(),
                surface_format,
                grid_width as u32,
                grid_height as u32,
            )
        });

        let max_texture_size = pixels.device().limits().max_texture_dimension_2d as usize;

        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &*window,
            Some(scale_factor),
            None,
            Some(max_texture_size),
        );
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [size.width, size.height],
            pixels_per_point: scale_factor,
        };
        let renderer = egui_wgpu::Renderer::new(
            pixels.device(),
            surface_format,
            RendererOptions {
                ..Default::default()
            },
        );

        Self {
            egui_ctx,
            egui_state,
            screen_descriptor,
            renderer,
            paint_jobs: Vec::new(),
            textures: egui::TexturesDelta::default(),

            pixels,
            bloom_effect,
            bloom_buffer: vec![0u8; grid_width * grid_height * 4],
            grid_width,
            grid_height,

            cursor_pos: (0.0, 0.0),
            left_down: false,
            right_down: false,
            scroll_delta: 0.0,
            egui_wants_pointer: false,

            frame_count: 0,
            next_tick: Instant::now(),
            frame_dur: Duration::from_secs_f64(1.0 / tick_rate),
            fps_window_start: Instant::now(),
            fps_frames_this_window: 0,
            fps: 0,
        }
    }

    /// Feed a raw winit event in. Tracks cursor/mouse/scroll and forwards to egui.
    /// Returns whether egui consumed the event.
    pub fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        let consumed = self.egui_state.on_window_event(window, event).consumed;
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = (position.x as f32, position.y as f32);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let pressed = *state == winit::event::ElementState::Pressed;
                match button {
                    winit::event::MouseButton::Left => self.left_down = pressed,
                    winit::event::MouseButton::Right => self.right_down = pressed,
                    _ => {}
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.scroll_delta += match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => *y,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 20.0,
                };
            }
            _ => {}
        }
        consumed
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.pixels.resize_surface(width, height).ok();
            self.screen_descriptor.size_in_pixels = [width, height];
        }
    }

    pub fn draw_frame(&mut self, window: &Window, sketch: &mut impl Simulation) {
        // fps measurement
        self.fps_frames_this_window += 1;
        let elapsed = self.fps_window_start.elapsed();
        if elapsed.as_secs_f32() >= 1.0 {
            self.fps = (self.fps_frames_this_window as f32 / elapsed.as_secs_f32()).round() as i32;
            self.fps_frames_this_window = 0;
            self.fps_window_start = Instant::now();
        }

        // fixed-rate tick timing + left/right alternation
        let now = Instant::now();
        let should_tick = now >= self.next_tick;
        let tick_left_to_right = self.frame_count % 2 == 0;
        if should_tick {
            self.frame_count += 1;
            self.next_tick += self.frame_dur;
            if self.next_tick < now {
                self.next_tick = now + self.frame_dur;
            }
        }

        let cursor_grid_pos = (!self.egui_wants_pointer)
            .then(|| self.pixels.window_pos_to_pixel(self.cursor_pos).ok())
            .flatten()
            .map(|(x, y)| (x as i32, y as i32));

        let input = FrameInput {
            cursor_grid_pos,
            left_down: self.left_down,
            right_down: self.right_down,
            scroll_delta: self.scroll_delta,
            should_tick,
            tick_left_to_right,
            fps: self.fps,
        };
        self.scroll_delta = 0.0;

        sketch.update(&input);

        let frame = self.pixels.frame_mut();
        sketch.generate_texture(frame, &mut self.bloom_buffer);

        if let Some(bloom) = &self.bloom_effect {
            bloom.upload(self.pixels.queue(), &self.bloom_buffer);
        }

        let raw_input = self.egui_state.take_egui_input(window);
        let output = self.egui_ctx.run_ui(raw_input, |ui| sketch.ui(ui, &input));
        self.textures.append(output.textures_delta);
        self.egui_state
            .handle_platform_output(window, output.platform_output);
        self.paint_jobs = self
            .egui_ctx
            .tessellate(output.shapes, output.pixels_per_point);
        self.egui_wants_pointer = self.egui_ctx.wants_pointer_input();

        let bloom_viewport = self.compute_bloom_viewport(window);

        let Framework {
            pixels,
            bloom_effect,
            renderer,
            paint_jobs,
            textures,
            screen_descriptor,
            ..
        } = self;

        let render_result =
            pixels.render_with(|encoder, render_target, context: &PixelsContext| {
                context.scaling_renderer.render(encoder, render_target);

                if let Some(bloom) = bloom_effect {
                    bloom.render(encoder, render_target, bloom_viewport);
                }

                for (id, delta) in &textures.set {
                    renderer.update_texture(&context.device, &context.queue, *id, delta);
                }
                renderer.update_buffers(
                    &context.device,
                    &context.queue,
                    encoder,
                    paint_jobs,
                    screen_descriptor,
                );
                let mut rpass = encoder
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("egui"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: render_target,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        ..Default::default()
                    })
                    .forget_lifetime();

                renderer.render(&mut rpass, paint_jobs, screen_descriptor);
                drop(rpass);

                for id in &textures.free {
                    renderer.free_texture(id);
                }
                textures.clear();

                Ok(())
            });
        if render_result.is_err() {
            return;
        }

        window.request_redraw();
    }

    fn compute_bloom_viewport(&self, window: &Window) -> (f32, f32, f32, f32) {
        let size = window.inner_size();
        let (sw, sh) = (size.width as f32, size.height as f32);
        let grid_aspect = self.grid_width as f32 / self.grid_height as f32;
        let (w, h) = if sw / sh > grid_aspect {
            (sh * grid_aspect, sh)
        } else {
            (sw, sw / grid_aspect)
        };
        ((sw - w) * 0.5, (sh - h) * 0.5, w, h)
    }

    pub fn ctx(&self) -> &egui::Context {
        &self.egui_ctx
    }
}
