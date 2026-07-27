use bitflags::bitflags;
use macroquad::color::Color;
use std::sync::LazyLock;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

bitflags! {
    #[derive(Debug,Clone,Copy,PartialEq )]
    pub struct Behavior: u16 {
        const FALLS = 1<<0; // Obeys gravity generally. Will fall down
        const FLOWS = 1 << 1; // flows sideways when blocked
        const STATIC = 1 << 2; // doesn't move
        const GRANULAR = 1 <<3; // Piles diagonally when blocked
        const RISES = 1<<4;

        const MELTABLE = 1<<5; // Destoryed by lava
        const FLAMMABLE = 1 <<6;

        const SAND = Self::FALLS.bits() | Self::GRANULAR.bits();
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
}
impl MaterialID {
    /// Returns a `MaterialProperties` struct of the properties associated with the materialID
    ///
    /// # Examples
    /// ```
    /// let x = MaterialID::Empty
    /// println!("{}",x.properties().desnsity)
    /// ```
    pub fn properties(self) -> &'static MaterialProperties {
        return &MATERIAL_TABLE[self as usize];
    }
}

pub static MATERIAL_TABLE: LazyLock<[MaterialProperties; 15]> = LazyLock::new(|| {
    [
        /* Empty */
        MaterialProperties {
            behavior: Behavior::empty(),
            density: 0.0,
            color: Color::new(0.0, 0.0, 0.0, 1.0),
            flow_distance: 0,
            lava_resistance: 0.0,
            flammability: 0.0,
        },
        /* Sand  */
        MaterialProperties {
            behavior: Behavior::SAND | Behavior::MELTABLE,
            density: 1.5,
            color: Color::new(0.96, 0.82, 0.45, 1.0),
            flow_distance: 0,
            lava_resistance: 0.8,
            flammability: 0.0,
        },
        /* Stone */
        MaterialProperties {
            behavior: Behavior::STATIC | Behavior::MELTABLE,
            density: 3.0,
            color: Color::new(0.5, 0.5, 0.5, 1.0),
            flow_distance: 0,
            lava_resistance: 0.05,
            flammability: 0.0,
        },
        /* Water */
        MaterialProperties {
            behavior: Behavior::LIQUID,
            density: 1.0,
            color: Color::new(0.1, 0.45, 0.82, 1.0),
            flow_distance: 5,
            lava_resistance: 1.0,
            flammability: 0.0,
        },
        /* Slime */
        MaterialProperties {
            behavior: Behavior::LIQUID,
            density: 1.3,
            color: Color::new(0.8, 0.3, 0.8, 1.0),
            flow_distance: 3,
            lava_resistance: 1.0,
            flammability: 0.0,
        },
        /* Salt */
        MaterialProperties {
            behavior: Behavior::SAND | Behavior::MELTABLE,
            density: 1.5,
            color: Color::new(0.9, 0.9, 1.0, 1.0),
            flow_distance: 3,
            lava_resistance: 0.1,
            flammability: 0.0,
        },
        /* Salt Water */
        MaterialProperties {
            behavior: Behavior::LIQUID,
            density: 1.1,
            color: Color::new(0.43, 0.77, 0.8, 1.0),
            flow_distance: 3,
            lava_resistance: 1.0,
            flammability: 0.0,
        },
        /* Lava */
        MaterialProperties {
            behavior: Behavior::LIQUID,
            density: 1.1,
            color: Color::new(0.95, 0.7, 0.0, 1.0),
            flow_distance: 1,
            lava_resistance: 1.0,
            flammability: 0.0,
        },
        /* Steam */
        MaterialProperties {
            behavior: Behavior::GAS,
            density: 0.1,
            color: Color::new(0.9, 0.9, 0.9, 1.0),
            flow_distance: 10,
            lava_resistance: 1.0,
            flammability: 0.0,
        },
        /* Dirt */
        MaterialProperties {
            behavior: Behavior::SAND | Behavior::MELTABLE,
            density: 1.7,
            color: Color::new(0.35, 0.23, 0.16, 1.0),
            flow_distance: 10,
            lava_resistance: 0.4,
            flammability: 0.0,
        },
        /* Snow */
        MaterialProperties {
            behavior: Behavior::SAND | Behavior::MELTABLE,
            density: 1.7,
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            flow_distance: 10,
            lava_resistance: 1.0,
            flammability: 0.0,
        },
        /* Oil */
        MaterialProperties {
            behavior: Behavior::LIQUID | Behavior::FLAMMABLE,
            density: 0.9,
            color: Color::new(0.14, 0.1, 0.05, 1.0),
            flow_distance: 3,
            lava_resistance: 0.0,
            flammability: 1.0,
        },
        /* Wood */
        MaterialProperties {
            behavior: Behavior::STATIC | Behavior::FLAMMABLE,
            density: 1.7,
            color: Color::new(0.18, 0.12, 0.0, 1.0),
            flow_distance: 10,
            lava_resistance: 1.0,
            flammability: 0.5,
        },
        /* Fire */
        MaterialProperties {
            behavior: Behavior::GAS,
            density: 0.1,
            color: Color::new(1.0, 0.5, 0.0, 1.0),
            flow_distance: 1,
            lava_resistance: 0.0,
            flammability: 0.0,
        },
        /* Smoke */
        MaterialProperties {
            behavior: Behavior::GAS,
            density: 0.1,
            color: Color::new(0.4, 0.4, 0.4, 1.0),
            flow_distance: 2,
            lava_resistance: 1.0,
            flammability: 0.0,
        },
    ]
});

#[derive(Debug)]
pub struct MaterialProperties {
    pub behavior: Behavior,
    pub density: f32,
    pub color: Color,

    pub flow_distance: u8,
    pub lava_resistance: f32,

    pub flammability: f32,
}
