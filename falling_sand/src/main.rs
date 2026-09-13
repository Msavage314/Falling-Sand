mod cell;
mod config;
mod explosion;
mod marching_squares;
mod materials;
mod particle;
mod reaction;
mod render;
mod rng;
mod simulation;
mod stains;
mod ui;
use materials::MaterialID;
use sand_render::{
    framework::Framework,
    simulation::{FrameInput, Simulation},
};
use simulation::Grid;
use std::sync::Arc;
use ui::UiState;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

/// Stores the sand simulation logic and user interface components.
struct SandSimulation {
    grid: Grid,
    ui: UiState,
}

impl Simulation for SandSimulation {
    fn update(&mut self, input: &FrameInput) {
        self.ui.set_fps(input.fps);
        self.ui.refresh_cache(&mut self.grid);

        if let Some((x, y)) = input.cursor_grid_pos {
            self.ui.current_cell = Some(self.grid.get(x, y));
        } else {
            self.ui.current_cell = None;
        };
        if let Some((gx, gy)) = input.cursor_grid_pos {
            if input.left_down {
                self.grid.draw_brush(gx, gy, self.ui.radius, self.ui.active);
            }
            if input.right_down {
                self.grid
                    .draw_brush(gx, gy, self.ui.radius, MaterialID::Empty);
            }
        }
        if input.scroll_delta != 0.0 {
            self.ui.radius = (self.ui.radius + input.scroll_delta.signum() as i32).clamp(1, 50);
        }
        if input.should_tick && self.ui.playing {
            self.grid.update(input.tick_left_to_right); // or alternate left/right as before
        }
    }

    fn generate_texture(&mut self, frame: &mut [u8], emissive: &mut [u8]) {
        render::draw(frame, emissive, &self.grid, self.ui.draw_bloom);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _input: &FrameInput) {
        self.ui.draw(ui, &mut self.grid);
    }
}

struct App {
    window: Option<Arc<Window>>,
    framework: Option<Framework>,
    simulation: SandSimulation,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("Falling Sand"))
                .unwrap(),
        );
        self.framework = Some(Framework::new(
            window.clone(),
            config::WIDTH,
            config::HEIGHT,
            config::FPS_TARGET,
            true,
        ));
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = &self.window else { return };
        if let Some(fw) = &mut self.framework {
            fw.handle_event(window, &event);
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(fw) = &mut self.framework {
                    fw.resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(fw) = &mut self.framework {
                    fw.draw_frame(window, &mut self.simulation);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Poll);
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
impl App {
    fn new(simulation: SandSimulation) -> Self {
        return Self {
            window: None,
            framework: None,
            simulation: simulation,
        };
    }
}
fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new(SandSimulation {
        grid: Grid::new(
            config::WIDTH,
            config::HEIGHT,
            MaterialID::DenseRock,
            config::CHUNK_SIZE,
        ),
        ui: UiState::new(),
    });
    event_loop.run_app(&mut app).unwrap();
}
