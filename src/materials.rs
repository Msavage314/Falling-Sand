use bitflags::bitflags;
use macroquad::color::Color;
use std::sync::LazyLock;
use strum_macros::EnumIter;
pub const MATERIAL_COUNT: usize = 19;
bitflags! {
    #[derive(Debug,Clone,Copy,PartialEq )]
    pub struct Behavior: u16 {
        const FALLS = 1<<0; // Obeys gravity generally. Will fall down
        const FLOWS = 1 << 1; // flows sideways when blocked
        const STATIC = 1 << 2; // doesn't move
        const GRANULAR = 1 <<3; // Piles diagonally when blocked
        const RISES = 1<<4;
        const SAND = 1<<10; // Present in sand

        const MELTABLE = 1<<5; // Destroyed by lava
        const FLAMMABLE = 1 <<6;

        const CORRODIBLE = 1<<7;

        const PERMEABLE = 1<<8; // Can be given the wet stain

        const HOT = 1<<9; // Causes the wet stain to disappear and causes water to evaporate

        const POWDER = Self::FALLS.bits() | Self::GRANULAR.bits() | Self::SAND.bits();
        const LIQUID = Self::FALLS.bits() | Self::FLOWS.bits() | Self::GRANULAR.bits();
        const GAS = Self::RISES.bits() |Self::FLOWS.bits();


    }
}

/// Stores a material ID that represents a specific material
#[derive(Debug, Clone, Copy, PartialEq, EnumIter, Eq, Hash)]
pub enum MaterialID {
    Empty,
    Sand,
    Stone,
    Water,
    Slime,
    Salt,
    SaltWater,
    Lava,
    Steam,
    Dirt,
    Snow,
    Oil,
    Wood,
    Fire,
    Smoke,
    Acid,
    DenseRock,
    FlammableGas,
    Rainbow,
}
impl MaterialID {
    /// Returns a `MaterialProperties` struct of the properties associated with the materialID
    ///
    /// # Examples
    /// ```
    /// let x = MaterialID::Empty
    /// println!("{}",x.properties().density)
    /// ```
    pub fn properties(self) -> &'static MaterialProperties {
        return &MATERIAL_TABLE[self as usize];
    }
}

pub static MATERIAL_TABLE: LazyLock<[MaterialProperties; MATERIAL_COUNT]> = LazyLock::new(|| {
    [
        /* Empty */
        MaterialProperties {
            behavior: Behavior::empty(),
            density: 0.5,
            color: |_x, _y| Color::new(0.05, 0.05, 0.05, 1.0),
            ..Default::default()
        },
        /* Sand  */
        MaterialProperties {
            behavior: Behavior::POWDER | Behavior::MELTABLE | Behavior::CORRODIBLE,
            density: 1.5,
            color: |x, y| vary_color(Color::new(0.96, 0.82, 0.45, 1.0), 0.05, x, y),
            lava_resistance: 0.8,
            flammability: 0.0,
            acid_resistance: 0.2,
            wake_chance: 1.0,
            ..Default::default()
        },
        /* Stone */
        MaterialProperties {
            behavior: Behavior::STATIC | Behavior::MELTABLE | Behavior::CORRODIBLE,
            density: 3.0,
            color: |x, y| vary_color(Color::new(0.5, 0.5, 0.5, 1.0), 0.05, x, y),
            lava_resistance: 0.05,
            acid_resistance: 0.7,
            ..Default::default()
        },
        /* Water */
        MaterialProperties {
            behavior: Behavior::LIQUID | Behavior::CORRODIBLE,
            density: 1.0,
            color: |_x, _y| Color::new(0.1, 0.45, 0.82, 1.0),
            flow_distance: 5,
            acid_resistance: 0.5,
            ..Default::default()
        },
        /* Slime */
        MaterialProperties {
            behavior: Behavior::LIQUID | Behavior::MELTABLE | Behavior::CORRODIBLE,
            density: 1.3,
            color: |x, y| vary_color(Color::new(0.8, 0.3, 0.8, 1.0), 0.05, x, y),
            flow_distance: 3,
            lava_resistance: 1.0,
            acid_resistance: 0.9,
            ..Default::default()
        },
        /* Salt */
        MaterialProperties {
            behavior: Behavior::POWDER | Behavior::MELTABLE | Behavior::CORRODIBLE,
            density: 1.5,
            color: |x, y| vary_color(Color::new(0.9, 0.9, 1.0, 1.0), 0.05, x, y),
            lava_resistance: 0.1,
            acid_resistance: 0.7,
            wake_chance: 0.4,
            ..Default::default()
        },
        /* Salt Water */
        MaterialProperties {
            behavior: Behavior::LIQUID | Behavior::CORRODIBLE,
            density: 1.1,
            color: |_x, _y| Color::new(0.43, 0.77, 0.8, 1.0),
            flow_distance: 3,
            acid_resistance: 0.5,
            ..Default::default()
        },
        /* Lava */
        MaterialProperties {
            behavior: Behavior::LIQUID | Behavior::CORRODIBLE | Behavior::HOT,
            density: 1.1,
            color: |_x, _y| Color::new(0.95, 0.7, 0.0, 1.0),
            flow_distance: 1,
            acid_resistance: 0.3,
            cools_to: MaterialID::Stone,
            ..Default::default()
        },
        /* Steam */
        MaterialProperties {
            behavior: Behavior::GAS | Behavior::CORRODIBLE,
            density: 0.1,
            color: |_x, _y| Color::new(0.9, 0.9, 0.9, 1.0),
            flow_distance: 6,
            acid_resistance: 0.1,
            ..Default::default()
        },
        /* Dirt */
        MaterialProperties {
            behavior: Behavior::POWDER
                | Behavior::MELTABLE
                | Behavior::CORRODIBLE
                | Behavior::PERMEABLE,
            density: 1.7,
            color: |x, y| vary_color(Color::new(0.35, 0.23, 0.16, 1.0), 0.05, x, y),
            lava_resistance: 0.4,
            acid_resistance: 0.5,
            wake_chance: 0.3,
            ..Default::default()
        },
        /* Snow */
        MaterialProperties {
            behavior: Behavior::POWDER | Behavior::MELTABLE | Behavior::CORRODIBLE,
            density: 1.7,
            color: |x, y| vary_color(Color::new(0.9, 1.0, 1.0, 0.9), 0.02, x, y),
            lava_resistance: 1.0,
            acid_resistance: 0.0,
            wake_chance: 0.05,
            ..Default::default()
        },
        /* Oil */
        MaterialProperties {
            behavior: Behavior::LIQUID | Behavior::FLAMMABLE | Behavior::CORRODIBLE,
            density: 0.9,
            color: |_x, _y| Color::new(0.14, 0.1, 0.05, 1.0),
            flow_distance: 3,
            flammability: 0.1,
            burn_intensity: 2.0,
            burn_time: 3.0,
            acid_resistance: 0.1,
            ..Default::default()
        },
        /* Wood */
        MaterialProperties {
            behavior: Behavior::STATIC
                | Behavior::FLAMMABLE
                | Behavior::CORRODIBLE
                | Behavior::PERMEABLE,
            density: 1.7,
            color: |_x, _y| Color::from_rgba(74, 61, 38, 255),
            flammability: 0.05,
            burn_intensity: 1.0,
            burn_time: 5.0,
            acid_resistance: 0.2,
            ..Default::default()
        },
        /* Fire */
        MaterialProperties {
            behavior: Behavior::GAS | Behavior::HOT,
            density: 0.1,
            color: |_x, _y| Color::new(1.0, 0.5, 0.0, 1.0),
            flow_distance: 1,
            ..Default::default()
        },
        /* Smoke */
        MaterialProperties {
            behavior: Behavior::GAS | Behavior::CORRODIBLE,
            density: 0.1,
            color: |_x, _y| Color::new(0.4, 0.4, 0.4, 1.0),
            flow_distance: 2,
            acid_resistance: 0.0,
            ..Default::default()
        },
        /* Acid */
        MaterialProperties {
            behavior: Behavior::LIQUID,
            density: 0.9,
            color: |_x, _y| Color::new(0.14, 0.74, 0.31, 1.0),
            flow_distance: 4,
            ..Default::default()
        },
        /* DenseRock */
        MaterialProperties {
            behavior: Behavior::STATIC,
            density: 0.9,
            color: |x, y| vary_color(Color::new(0.2, 0.2, 0.2, 1.0), 0.05, x, y),
            ..Default::default()
        },
        /* FlammableGas */
        MaterialProperties {
            behavior: Behavior::GAS | Behavior::FLAMMABLE,
            density: 0.1,
            color: |_x, _y| Color::new(0.2, 0.34, 0.11, 1.0),
            flow_distance: 7,
            flammability: 1.0,
            burn_intensity: 5.0,
            burn_time: 0.2,
            ..Default::default()
        },
        /* Rainbow */
        MaterialProperties {
            behavior: Behavior::POWDER,
            density: 2.0,
            color: |x, y| vary_color(rainbow_color(x, y), 0.1, x, y),
            ..Default::default()
        },
    ]
});

#[derive(Debug)]
pub struct MaterialProperties {
    pub behavior: Behavior,
    pub density: f32,
    pub color: fn(i32, i32) -> Color,

    pub flow_distance: u8,
    pub lava_resistance: f32,

    pub flammability: f32, // Chance of igniting when next to other materials burning
    pub burn_time: f32,    // How long this material burns once ignited
    pub burn_intensity: f32, // peak flame intensity

    pub acid_resistance: f32, // chance of dissolving in acid

    pub wake_chance: f32, // chance that the cell comes "awake"

    pub cools_to: MaterialID, // What the material turns into upon contact with something cool. Defaults to empty. For example, lava turns to stone
}

impl Default for MaterialProperties {
    fn default() -> Self {
        Self {
            behavior: Behavior::empty(),
            density: 0.0,
            color: |_x, _yy| Color::new(0.0, 0.0, 0.0, 1.0),
            flow_distance: 0,
            lava_resistance: 0.0,
            flammability: 0.0,
            burn_intensity: 0.0,
            burn_time: 0.0,
            acid_resistance: 0.0,
            wake_chance: 1.0,
            cools_to: MaterialID::Empty,
        }
    }
}

pub fn hsv_to_color(h: f32, s: f32, v: f32) -> Color {
    let c = v * s;
    let hp = h / 60.0;
    let x = c * (1.0 - (hp % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if hp >= 0.0 && hp < 1.0 {
        (c, x, 0.0)
    } else if hp >= 1.0 && hp < 2.0 {
        (x, c, 0.0)
    } else if hp >= 2.0 && hp < 3.0 {
        (0.0, c, x)
    } else if hp >= 3.0 && hp < 4.0 {
        (0.0, x, c)
    } else if hp >= 4.0 && hp < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    Color::new(r + m, g + m, b + m, 1.0)
}
pub fn rainbow_color(x: i32, _y: i32) -> Color {
    return hsv_to_color(((12 * x) % 360) as f32, 0.8, 0.7);
}

pub fn vary_color(base: Color, amount: f32, x: i32, y: i32) -> Color {
    let jitter = hash_jitter(x, y, amount);
    Color::new(
        (base.r + jitter).clamp(0.0, 1.0),
        (base.g + jitter).clamp(0.0, 1.0),
        (base.b + jitter).clamp(0.0, 1.0),
        base.a,
    )
}

fn hash_jitter(x: i32, y: i32, amount: f32) -> f32 {
    // simple integer hash (xor shift-ish), deterministic per coordinate
    let mut h = (x as u32).wrapping_mul(374761393) ^ (y as u32).wrapping_mul(668265263);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^= h >> 16;

    // map hash to [0, 1)
    let normalized = (h as f32) / (u32::MAX as f32);

    // map to [-amount, amount]
    (normalized * 2.0 - 1.0) * amount
}

pub fn wood_color(x: i32, y: i32) -> Color {
    let mut c = Color::new(0.60, 0.40, 0.18, 1.0);

    let plank_width = 12;
    let plank = x.div_euclid(plank_width);
    let local = x.rem_euclid(plank_width);

    // Each plank has a different tint
    let plank_offset = hash_jitter(plank, 0, 0.08);

    // Long grain
    let grain = ((y as f32 * 0.18).sin()) * 0.03 + hash_jitter(x / 6, y / 2, 0.03);

    let mut brightness = plank_offset + grain;

    // Dark seams
    if local == 0 || local == plank_width - 1 {
        brightness -= 0.15;
    }

    c.r = (c.r + brightness).clamp(0.0, 1.0);
    c.g = (c.g + brightness).clamp(0.0, 1.0);
    c.b = (c.b + brightness).clamp(0.0, 1.0);

    c
}
