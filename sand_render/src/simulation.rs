/// Contains information about all inputs required for that frame
pub struct FrameInput {
    /// Position of the cursor on the grid
    pub cursor_grid_pos: Option<(i32, i32)>,
    /// whether the left mouse button is down
    pub left_down: bool,
    /// whether the right mouse button is down
    pub right_down: bool,
    pub scroll_delta: f32,
    /// false if the simulation if paused
    pub should_tick: bool,
    /// decides which way the grid simulation should tick. It alternates each frame in order to prevent one side
    /// always being updated first
    pub tick_left_to_right: bool,
    pub fps: i32,
}

pub trait Simulation {
    fn update(&mut self, input: &FrameInput);
    /// called to generate a texture buffer representing a frame of the simulation
    fn generate_texture(&mut self, frame: &mut [u8], emissive: &mut [u8]);
    /// returns the ui
    fn ui(&mut self, ui: &mut egui::Ui, input: &FrameInput);
}
