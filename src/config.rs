use crate::materials::MaterialID;
/// Materials which the user is allowed to pick as the edge of the simulation
/// `DenseRock` results in sand stopping at the edge, whereas `Empty` makes it fall offscreen
pub const BORDER_OPTIONS: [MaterialID; 2] = [MaterialID::Empty, MaterialID::DenseRock];
