#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StainKind {
    Burning,
    Wet,
}

#[derive(Debug, Clone, Copy)]
pub struct Stain {
    pub kind: StainKind,
    pub intensity: f32, // 0.0-1.0
    pub timer: f32,     // remaining lifetime
}
