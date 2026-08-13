use crate::materials::MaterialID;

/// Materials which the user is allowed to pick as the edge of the simulation
/// `DenseRock` results in sand stopping at the edge, whereas `Empty` makes it fall offscreen
pub const BORDER_OPTIONS: [MaterialID; 2] = [MaterialID::Empty, MaterialID::DenseRock];

pub const WIDTH: usize = 400;
pub const HEIGHT: usize = 300;

// Cells per tick per tick of acceleration
pub const GRAVITY: f32 = 0.1;
