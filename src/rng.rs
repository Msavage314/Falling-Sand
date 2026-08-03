use macroquad;

pub fn chance(p: f32) -> bool {
    return macroquad::rand::gen_range(0.0, 1.0) < p;
}

// /// IMPORTANT:
// /// This is actually per 60 frames, as it doesn't adjust for low frame rates
// pub fn chance_per_second(p: f32) -> bool {
//     return macroquad::rand::gen_range(0.0, 1.0) < (p / 60.0);
// }
