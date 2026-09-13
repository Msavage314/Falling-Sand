use std::sync::LazyLock;

use crate::cell::Color;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub const MATERIAL_COUNT: usize = 29;

bitflags! {
    #[derive(Debug,Clone,Copy,PartialEq )]
    pub struct Behavior: u16 {
        const FALLS = 1<<0; // Obeys gravity generally. Will fall down
        const FLOWS = 1 << 1; // flows sideways when blocked
        const STATIC = 1 << 2; // doesn't move
        const GRANULAR = 1 <<3; // Piles diagonally when blocked
        const RISES = 1<<4; // Gases do this. Like falling but up
        const SAND = 1<<10; // Present in sand. Prevents Liquids from also satisfying powder

        const MELTABLE = 1<<5; // Destroyed by lava
        const FLAMMABLE = 1 <<6;

        const CORRODIBLE = 1<<7;

        const PERMEABLE = 1<<8; // Can be given the wet stain

        const HOT = 1<<9; // Causes the wet stain to disappear and causes water to evaporate

        const BLOOM = 1<<10; // has glowing effect

        const POWDER = Self::FALLS.bits() | Self::GRANULAR.bits() | Self::SAND.bits();
        const LIQUID = Self::FALLS.bits() | Self::FLOWS.bits() | Self::GRANULAR.bits();
        const GAS = Self::RISES.bits() | Self::FLOWS.bits();
    }
}

/// Stores a material ID that represents a specific material
#[derive(Debug, Clone, Copy, PartialEq, EnumIter, Eq, Hash, Serialize, Deserialize)]
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
    Coal,
    ToxicSludge,
    Gunpowder,
    Nitro,
    TNT,
    Methane,
    Dynamite,
    Experimental,
    Spout,
    Volcano,
}
impl MaterialID {
    /// Returns a `MaterialProperties` struct of the properties associated with the materialID
    ///
    /// # Examples
    /// ```
    /// let x = MaterialID::Empty
    /// println!("{}",x.properties().density)
    /// ```
    ///
    pub fn compute_properties(self) -> MaterialProperties {
        match self {
            MaterialID::Empty => MaterialProperties {
                behavior: Behavior::empty(),
                density: 0.5,
                color: |_x, _y| Color::new(0.05, 0.05, 0.05, 1.0),
                ..Default::default()
            },
            MaterialID::Sand => MaterialProperties {
                behavior: Behavior::POWDER | Behavior::MELTABLE | Behavior::CORRODIBLE,
                density: 1.5,
                color: |x, y| vary_color(Color::new(0.96, 0.82, 0.45, 1.0), 0.05, x, y),
                lava_resistance: 0.8,
                flammability: 0.0,
                acid_resistance: 0.2,
                wake_chance: 1.0,
                ..Default::default()
            },
            MaterialID::Stone => MaterialProperties {
                behavior: Behavior::STATIC
                    | Behavior::MELTABLE
                    | Behavior::CORRODIBLE
                    | Behavior::PERMEABLE,
                density: 3.0,
                color: |x, y| vary_color(Color::new(0.5, 0.5, 0.5, 1.0), 0.05, x, y),
                lava_resistance: 0.1,
                acid_resistance: 0.7,
                permeability: 5.0,
                ..Default::default()
            },
            MaterialID::Water => MaterialProperties {
                behavior: Behavior::LIQUID | Behavior::CORRODIBLE,
                density: 1.0,
                color: |x, y| vary_color(Color::new(0.1, 0.45, 0.82, 1.0), 0.01, x, y),
                flow_distance: 5,
                acid_resistance: 0.5,
                ..Default::default()
            },
            MaterialID::Slime => MaterialProperties {
                behavior: Behavior::LIQUID | Behavior::MELTABLE | Behavior::CORRODIBLE,
                density: 1.3,
                color: |x, y| vary_color(Color::new(0.8, 0.3, 0.8, 1.0), 0.05, x, y),
                flow_distance: 3,
                lava_resistance: 1.0,
                acid_resistance: 0.9,
                ..Default::default()
            },
            MaterialID::Salt => MaterialProperties {
                behavior: Behavior::POWDER | Behavior::MELTABLE | Behavior::CORRODIBLE,
                density: 1.5,
                color: |x, y| vary_color(Color::new(0.9, 0.9, 1.0, 1.0), 0.05, x, y),
                lava_resistance: 0.1,
                acid_resistance: 0.7,
                wake_chance: 0.4,
                ..Default::default()
            },
            MaterialID::SaltWater => MaterialProperties {
                behavior: Behavior::LIQUID | Behavior::CORRODIBLE,
                density: 1.1,
                color: |_x, _y| Color::new(0.43, 0.77, 0.8, 1.0),
                flow_distance: 3,
                acid_resistance: 0.5,
                ..Default::default()
            },
            MaterialID::Lava => MaterialProperties {
                behavior: Behavior::LIQUID | Behavior::CORRODIBLE | Behavior::HOT,
                density: 1.1,
                color: |_x, _y| Color::new(0.95, 0.7, 0.0, 1.0),
                flow_distance: 1,
                acid_resistance: 0.3,
                cools_to: MaterialID::Stone,
                bloom: 1.0,
                ..Default::default()
            },
            MaterialID::Steam => MaterialProperties {
                behavior: Behavior::GAS | Behavior::CORRODIBLE,
                density: 0.1,
                color: |_x, _y| Color::new(0.9, 0.9, 0.9, 1.0),
                flow_distance: 6,
                acid_resistance: 0.1,
                bloom: 1.0,
                ..Default::default()
            },
            MaterialID::Dirt => MaterialProperties {
                behavior: Behavior::POWDER
                    | Behavior::MELTABLE
                    | Behavior::CORRODIBLE
                    | Behavior::PERMEABLE,
                density: 1.7,
                color: |x, y| vary_color(Color::new(0.35, 0.23, 0.16, 1.0), 0.05, x, y),
                lava_resistance: 0.4,
                acid_resistance: 0.5,
                wake_chance: 0.3,
                permeability: 12.0,
                ..Default::default()
            },
            MaterialID::Snow => MaterialProperties {
                behavior: Behavior::POWDER | Behavior::MELTABLE | Behavior::CORRODIBLE,
                density: 1.7,
                color: |x, y| vary_color(Color::new(0.9, 1.0, 1.0, 0.9), 0.02, x, y),
                lava_resistance: 1.0,
                acid_resistance: 0.0,
                wake_chance: 0.05,
                ..Default::default()
            },
            MaterialID::Oil => MaterialProperties {
                behavior: Behavior::LIQUID | Behavior::FLAMMABLE | Behavior::CORRODIBLE,
                density: 0.9,
                color: |_x, _y| Color::new(0.14, 0.1, 0.05, 1.0),
                flow_distance: 3,
                flammability: 0.5,
                burn_intensity: 2.0,
                burn_time: 5.0,
                acid_resistance: 0.1,
                ..Default::default()
            },
            MaterialID::Wood => MaterialProperties {
                behavior: Behavior::STATIC
                    | Behavior::FLAMMABLE
                    | Behavior::CORRODIBLE
                    | Behavior::PERMEABLE,
                density: 1.7,
                color: |_x, _y| Color::from_rgba(74, 61, 38, 255),
                flammability: 0.05,
                burn_intensity: 1.0,
                burn_time: 7.0,
                acid_resistance: 0.2,
                permeability: 20.0,
                ..Default::default()
            },
            MaterialID::Fire => MaterialProperties {
                behavior: Behavior::GAS | Behavior::HOT,
                density: 0.1,
                color: |_x, _y| Color::new(1.0, 0.5, 0.0, 1.0),
                flow_distance: 1,
                bloom: 1.0,
                ..Default::default()
            },
            MaterialID::Smoke => MaterialProperties {
                behavior: Behavior::GAS | Behavior::CORRODIBLE,
                density: 0.1,
                color: |_x, _y| Color::new(0.4, 0.4, 0.4, 1.0),
                flow_distance: 2,
                acid_resistance: 0.0,
                bloom: 1.0,
                ..Default::default()
            },
            MaterialID::Acid => MaterialProperties {
                behavior: Behavior::LIQUID,
                density: 0.9,
                color: |_x, _y| Color::new(0.14, 0.74, 0.31, 1.0),
                flow_distance: 4,
                bloom: 0.5,
                ..Default::default()
            },
            MaterialID::DenseRock => MaterialProperties {
                behavior: Behavior::STATIC,
                density: 0.9,
                color: |x, y| vary_color(Color::new(0.2, 0.2, 0.2, 1.0), 0.05, x, y),
                explosion_resistance: 1000.0,
                ..Default::default()
            },
            MaterialID::FlammableGas => MaterialProperties {
                behavior: Behavior::GAS | Behavior::FLAMMABLE,
                density: 0.1,
                color: |_x, _y| Color::new(0.2, 0.34, 0.11, 1.0),
                flow_distance: 7,
                flammability: 1.0,
                burn_intensity: 5.0,
                burn_time: 0.2,
                bloom: 0.6,
                ..Default::default()
            },
            MaterialID::Rainbow => MaterialProperties {
                behavior: Behavior::POWDER,
                density: 2.0,
                color: |x, y| vary_color(rainbow_color(x, y), 0.1, x, y),
                bloom: 0.3,
                ..Default::default()
            },
            MaterialID::Coal => MaterialProperties {
                behavior: Behavior::POWDER | Behavior::FLAMMABLE | Behavior::PERMEABLE,
                density: 2.0,
                color: |x, y| vary_color(Color::new(0.1, 0.1, 0.1, 1.0), 0.1, x, y),
                flammability: 0.004,
                burn_intensity: 1.0,
                burn_time: 10.0,
                wake_chance: 0.1,
                permeability: 2.0,
                ..Default::default()
            },
            MaterialID::ToxicSludge => MaterialProperties {
                behavior: Behavior::LIQUID | Behavior::CORRODIBLE,
                density: 0.9,
                color: |_x, _y| Color::from_rgba(107, 145, 19, 255),
                flow_distance: 5,
                acid_resistance: 0.4,
                bloom: 1.0,
                ..Default::default()
            },
            MaterialID::Gunpowder => MaterialProperties {
                behavior: Behavior::POWDER | Behavior::CORRODIBLE,
                density: 5.5,
                color: |x, y| vary_color(Color::from_rgba(133, 133, 133, 255), 0.1, x, y),
                flow_distance: 5,
                acid_resistance: 0.4,
                ..Default::default()
            },
            MaterialID::Nitro => MaterialProperties {
                behavior: Behavior::LIQUID | Behavior::CORRODIBLE,
                density: 1.0,
                color: |x, y| vary_color(Color::from_rgba(6, 92, 0, 255), 0.01, x, y),
                flow_distance: 6,
                acid_resistance: 0.4,
                ..Default::default()
            },
            MaterialID::TNT => MaterialProperties {
                behavior: Behavior::STATIC | Behavior::CORRODIBLE,
                density: 2.0,
                color: |x, y| vary_color(Color::from_rgba(92, 24, 0, 255), 0.01, x, y),
                acid_resistance: 0.4,
                ..Default::default()
            },
            MaterialID::Methane => MaterialProperties {
                behavior: Behavior::GAS | Behavior::FLAMMABLE,
                density: 0.1,
                color: |_x, _y| Color::from_rgba(135, 135, 135, 255),
                ..Default::default()
            },
            MaterialID::Dynamite => MaterialProperties {
                behavior: Behavior::POWDER | Behavior::CORRODIBLE,
                density: 2.0,
                color: |x, y| vary_color(Color::from_rgba(133, 67, 45, 255), 0.05, x, y),
                acid_resistance: 0.2,
                wake_chance: 0.2,
                ..Default::default()
            },
            MaterialID::Experimental => MaterialProperties {
                behavior: Behavior::POWDER,
                density: 10.0,
                color: |x, y| vary_color(Color::from_rgba(13, 255, 0, 0), 0.05, x, y),
                wake_chance: 1.0,
                bloom: 0.5,
                ..Default::default()
            },
            MaterialID::Spout => MaterialProperties {
                behavior: Behavior::STATIC,
                color: |_x, _y| Color::from_rgba(106, 189, 222, 255),
                ..Default::default()
            },
            MaterialID::Volcano => MaterialProperties {
                behavior: Behavior::STATIC,
                color: |_x, _y| Color::from_rgba(181, 109, 0, 255),
                ..Default::default()
            },
        }
    }
    #[inline]
    pub fn properties(self) -> MaterialProperties {
        MATERIAL_PROPERTIES[self as usize]
    }
}

#[derive(Debug, Clone, Copy)]
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
    pub explosion_resistance: f32,

    pub bloom: f32,

    pub permeability: f32, // 1/amount of wetness lost by transfer between material
}

impl Default for MaterialProperties {
    fn default() -> Self {
        Self {
            behavior: Behavior::empty(),
            density: 0.0,
            color: |_x, _yy| Color::new(0.0, 0.0, 0.0, 1.0),
            flow_distance: 1,
            lava_resistance: 0.0,
            flammability: 0.0,
            burn_intensity: 0.0,
            burn_time: 0.0,
            acid_resistance: 0.0,
            wake_chance: 1.0,
            cools_to: MaterialID::Empty,
            explosion_resistance: 1.0,
            bloom: 0.0,
            permeability: 1.0,
        }
    }
}

static MATERIAL_PROPERTIES: LazyLock<[MaterialProperties; MATERIAL_COUNT]> = LazyLock::new(|| {
    std::array::from_fn(|i| MaterialID::iter().nth(i).unwrap().compute_properties())
});

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

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn sand_is_falls_and_granular() {
        let props = MaterialID::Sand.properties();
        assert!(props.behavior.contains(Behavior::FALLS));
        assert!(props.behavior.contains(Behavior::GRANULAR));
    }
    #[test]
    fn water_denser_than_oil() {
        assert!(MaterialID::Water.properties().density > MaterialID::Oil.properties().density)
    }

    #[test]
    fn material_table_covers_all() {
        // would panic if a variant were missing a match arm (compile-time now, not runtime)
        for id in MaterialID::iter() {
            let _ = id.properties();
        }
    }
}
