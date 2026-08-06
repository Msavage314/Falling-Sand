use crate::materials::Behavior;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StainKind {
    Burning,
    Wet,
    Slimy,
}
impl StainKind {
    pub fn behavior(self) -> Behavior {
        match self {
            StainKind::Burning => Behavior::HOT,
            _ => Behavior::empty(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Stain {
    pub kind: StainKind,
    pub intensity: f32, // 0.0-1.0
    pub timer: f32,     // remaining lifetime
}
