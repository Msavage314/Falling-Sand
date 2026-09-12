use egui;
use egui_wgpu::RendererOptions;
use pixels::{Pixels, PixelsContext, wgpu};
use winit::{event::WindowEvent, window::Window};

/// Owns the egui rendering, textures etc stuff that is required to actually draw stuff
/// also handles raw events. Handles the lower level stuff, whereas UiState actually contains
/// code to draw the ui.
/// Egui requires egui context, window states and renderer objects which are stored in here
/// Holds one instance of each: `egui::Context`, `egui_winit::State`, `egui_wgpu::Renderer`,
/// basically what is required to draw the UI onto the screen.
pub struct Framework {
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    screen_descriptor: egui_wgpu::ScreenDescriptor,
    renderer: egui_wgpu::Renderer,
    paint_jobs: Vec<egui::ClippedPrimitive>,
    textures: egui::TexturesDelta,
}
impl Framework {
    pub fn new(
        window: &Window,
        width: u32,
        height: u32,
        scale_factor: f32,
        pixels: &Pixels,
    ) -> Self {
        let max_texture_size = pixels.device().limits().max_texture_dimension_2d as usize;

        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            window,
            Some(scale_factor),
            None,
            Some(max_texture_size),
        );
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: scale_factor,
        };
        let renderer = egui_wgpu::Renderer::new(
            pixels.device(),
            pixels.render_texture_format(),
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
        }
    }

    pub fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        return self.egui_state.on_window_event(window, event).consumed;
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.screen_descriptor.size_in_pixels = [width, height]
        }
    }
    pub fn ctx(&self) -> &egui::Context {
        &self.egui_ctx
    }
    pub fn prepare(&mut self, window: &Window, run_ui: impl FnMut(&mut egui::Ui)) {
        let raw_input = self.egui_state.take_egui_input(window);
        let output = self.egui_ctx.run_ui(raw_input, run_ui);
        // textures delta is output textures since last frame
        self.textures.append(output.textures_delta);
        self.egui_state
            .handle_platform_output(window, output.platform_output);
        self.paint_jobs = self
            .egui_ctx
            .tessellate(output.shapes, output.pixels_per_point)
    }

    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
        context: &PixelsContext,
    ) {
        // add new textures
        for (id, delta) in &self.textures.set {
            self.renderer
                .update_texture(&context.device, &context.queue, *id, delta);
        }
        self.renderer.update_buffers(
            &context.device,
            &context.queue,
            encoder,
            &self.paint_jobs,
            &self.screen_descriptor,
        );
        let mut rpass = encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: render_target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // draw on top of sand texture,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                ..Default::default()
            })
            .forget_lifetime();

        self.renderer
            .render(&mut rpass, &self.paint_jobs, &self.screen_descriptor);
        drop(rpass);
        for id in &self.textures.free {
            self.renderer.free_texture(id);
        }
        self.textures.clear();
    }
}
