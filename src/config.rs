use crate::materials::MaterialID;

/// Materials which the user is allowed to pick as the edge of the simulation
/// `DenseRock` results in sand stopping at the edge, whereas `Empty` makes it fall offscreen
pub const BORDER_OPTIONS: [MaterialID; 2] = [MaterialID::Empty, MaterialID::DenseRock];

/// In simulations pixels
pub const WIDTH: usize = 320;
/// In simulation pixels
pub const HEIGHT: usize = 320;

/// Must divide into width and height
pub const CHUNK_SIZE: usize = 16;

/// Cells per tick per tick of acceleration
pub const GRAVITY: f32 = 0.1;

/// Target fps for the simulation to run at. There is no delta t used in this simulation, so higher speeds will make the simulation feel faster

pub const FPS_TARGET: f64 = 60.0;
